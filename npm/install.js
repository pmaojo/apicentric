const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');
const os = require('os');

const PACKAGE_VERSION = "0.3.1"; // Sync with package.json
const BIN_DIR = path.join(__dirname, 'bin');
const BIN_NAME = process.platform === 'win32' ? 'apicentric.exe' : 'apicentric';
const BIN_PATH = path.join(BIN_DIR, BIN_NAME);

// Mapping to GitHub Release asset names
const PLATFORM_MAP = {
    'win32': 'windows',
    'darwin': 'macos',
    'linux': 'linux'
};

const ARCH_MAP = {
    'x64': 'x64',
    'arm64': 'arm64'
};

function getDownloadUrl() {
    const platform = os.platform();
    const arch = os.arch();

    const mappedPlatform = PLATFORM_MAP[platform];
    const mappedArch = ARCH_MAP[arch];

    if (!mappedPlatform || !mappedArch) {
        throw new Error(`Unsupported platform or architecture: ${platform}-${arch}`);
    }

    const ext = platform === 'win32' ? 'zip' : 'tar.gz';
    return `https://github.com/pmaojo/apicentric/releases/download/v${PACKAGE_VERSION}/apicentric-${mappedPlatform}-${mappedArch}.${ext}`;
}

function downloadFile(url, dest) {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(dest);
        https.get(url, (response) => {
            if (response.statusCode === 302 || response.statusCode === 301) {
                downloadFile(response.headers.location, dest).then(resolve).catch(reject);
                return;
            }
            response.pipe(file);
            file.on('finish', () => {
                file.close(resolve);
            });
        }).on('error', (err) => {
            fs.unlink(dest, () => {});
            reject(err);
        });
    });
}

async function install() {
    try {
        if (!fs.existsSync(BIN_DIR)) {
            fs.mkdirSync(BIN_DIR);
        }

        const url = getDownloadUrl();
        console.log(`Downloading Apicentric v${PACKAGE_VERSION} from ${url}...`);
        
        const archiveName = path.basename(url);
        const archivePath = path.join(BIN_DIR, archiveName);

        await downloadFile(url, archivePath);

        console.log('Extracting...');
        if (url.endsWith('.zip')) {
            // Windows usually has powershell or tar (newer builds)
            try {
                execSync(`tar -xf "${archivePath}" -C "${BIN_DIR}"`); 
            } catch (e) {
                 // Fallback if tar fails on windows, maybe use powershell expand-archive?
                 // For simplicity, assuming tar exists (Win10+) or user has tools.
                 // A thorough impl would use a library like 'adm-zip' but we want 0 deps.
                 console.warn("tar failed, trying PowerShell Expand-Archive...");
                 execSync(`powershell -command "Expand-Archive -Force '${archivePath}' '${BIN_DIR}'"`);
            }
        } else {
            execSync(`tar -xzf "${archivePath}" -C "${BIN_DIR}"`);
        }

        // Cleanup
        fs.unlinkSync(archivePath);

        // Verify
        if (fs.existsSync(BIN_PATH)) {
            if (process.platform !== 'win32') {
                fs.chmodSync(BIN_PATH, 0o755);
            }
            console.log('✅ Apicentric installed successfully!');
        } else {
            throw new Error('Binary not found after extraction');
        }

    } catch (error) {
        console.error('❌ Installation failed:', error.message);
        process.exit(1);
    }
}

install();
