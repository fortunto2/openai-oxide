#!/usr/bin/env node

import { spawnSync } from 'node:child_process'

const args = process.argv.slice(2)

if (args.length === 0) {
  console.error(
    'Usage: node scripts/npm-publish-with-skip.mjs <command> [args...]',
  )
  process.exit(1)
}

const result = spawnSync(args[0], args.slice(1), {
  stdio: 'pipe',
  encoding: 'utf8',
})

if (result.stdout) {
  process.stdout.write(result.stdout)
}

if (result.stderr) {
  process.stderr.write(result.stderr)
}

if (result.status === 0) {
  process.exit(0)
}

const combinedOutput = `${result.stdout ?? ''}\n${result.stderr ?? ''}`

if (isAlreadyPublishedError(combinedOutput)) {
  console.warn(
    'WARNING npm publish target is already published; skipping duplicate publish',
  )
  process.exit(0)
}

if (result.error) {
  console.error(result.error.message)
}

process.exit(result.status ?? 1)

function isAlreadyPublishedError(output) {
  return (
    /previously published versions/i.test(output) ||
    /cannot publish over (?:the )?previously published versions/i.test(output) ||
    /cannot publish over existing version/i.test(output)
  )
}
