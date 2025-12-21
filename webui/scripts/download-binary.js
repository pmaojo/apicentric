const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const BACKEND_DIR = path.join(__dirname, '../backend');
const BINARY_NAME = 'apicentric';
const BINARY_PATH = path.join(BACKEND_DIR, BINARY_NAME);
const CONFIG_FILE = 'apicentric.json';

// Ensure backend directory exists
if (!fs.existsSync(BACKEND_DIR)) {
  fs.mkdirSync(BACKEND_DIR, { recursive: true });
}

// Create default config if missing
const configPath = path.join(BACKEND_DIR, CONFIG_FILE);
if (!fs.existsSync(configPath)) {
  fs.writeFileSync(configPath, JSON.stringify({ simulator: { enabled: true } }));
}

// Skip if binary exists
if (fs.existsSync(BINARY_PATH)) {
  console.log('Backend binary already exists.');
  process.exit(0);
}

console.log('Downloading apicentric binary...');

// Determine asset name (Hardcoded for Linux x64 as Vercel uses Amazon Linux 2)
// Ideally this should detect OS, but for Vercel deployment we target Linux.
const ASSET_NAME = 'apicentric-linux-x64.tar.gz';
const REPO = 'pmaojo/apicentric';

// Function to download file
function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        download(response.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      response.pipe(file);
      file.on('finish', () => {
        file.close(resolve);
      });
    }).on('error', (err) => {
      fs.unlink(dest);
      reject(err);
    });
  });
}

async function main() {
  try {
    // Get latest release info
    const releaseUrl = `https://api.github.com/repos/${REPO}/releases/latest`;
    const releaseData = await new Promise((resolve, reject) => {
        https.get(releaseUrl, { headers: { 'User-Agent': 'Node.js' } }, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve(JSON.parse(data)));
        }).on('error', reject);
    });

    const asset = releaseData.assets.find(a => a.name === ASSET_NAME);
    if (!asset) {
      throw new Error(`Asset ${ASSET_NAME} not found in latest release.`);
    }

    const downloadUrl = asset.browser_download_url;
    const tarPath = path.join(BACKEND_DIR, 'apicentric.tar.gz');

    console.log(`Downloading from ${downloadUrl}...`);
    await download(downloadUrl, tarPath);

    console.log('Extracting...');
    execSync(`tar -xzf ${tarPath} -C ${BACKEND_DIR}`);
    fs.unlinkSync(tarPath);

    // Make executable
    fs.chmodSync(BINARY_PATH, '755');
    console.log('Download complete.');

  } catch (error) {
    console.error('Failed to download binary:', error);
    process.exit(1);
  }
}

main();
