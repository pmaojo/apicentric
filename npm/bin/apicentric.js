#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const os = require('os');

const BIN_NAME = os.platform() === 'win32' ? 'apicentric.exe' : 'apicentric';
const BIN_PATH = path.join(__dirname, BIN_NAME);

const child = spawn(BIN_PATH, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env
});

child.on('exit', (code) => {
  process.exit(code);
});

child.on('error', (err) => {
    if (err.code === 'ENOENT') {
        console.error(`\n❌ Apicentric binary not found at: ${BIN_PATH}`);
        console.error('   Please verify the installation or run "npm rebuild apicentric"');
    } else {
        console.error('❌ Failed to start apicentric:', err);
    }
    process.exit(1);
});
