#!/usr/bin/env node
const path = require('path');
const { execFileSync } = require('child_process');

const execname = 'sitefix';

try {
    execFileSync(
        path.join(__dirname, `../bin/${execname}${process.platform === 'win32' ? '.exe' : ''}`),
        process.argv.slice(2),
        {
            stdio: [process.stdin, process.stdout, process.stderr]
        }
    )
} catch (err) {
    //Purposefully ignore errors as the CLI should handle all the messaging etc.
}