#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import { setTimeout } from 'node:timers/promises'

const args = process.argv.slice(2)

if (args.length === 0) {
  console.error(
    'Usage: node scripts/npm-publish-with-skip.mjs <command> [args...]',
  )
  process.exit(1)
}

const defaultMaxRetries = 3
const defaultRetryDelayMs = 25_000
const maxRetries = parsePositiveInt(
  process.env.NPM_PUBLISH_MAX_RETRIES,
  defaultMaxRetries,
)
const retryDelayMs = parsePositiveInt(
  process.env.NPM_PUBLISH_RETRY_DELAY_MS,
  defaultRetryDelayMs,
)

const main = async () => {
  for (let attempt = 1; attempt <= maxRetries; attempt += 1) {
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

    if (
      isRateLimitError(combinedOutput) &&
      attempt < maxRetries
    ) {
      console.warn(
        `Too many requests (attempt ${attempt}/${maxRetries}); retrying after ${retryDelayMs /
          1000}s`,
      )
      await setTimeout(retryDelayMs)
      continue
    }

    if (isRateLimitError(combinedOutput)) {
      console.error(
        'Rate limit persisted after retries; refusing to continue to avoid publishing duplicates.',
      )
    }

    if (isAuthError(combinedOutput)) {
      console.error(
        [
          '',
          'npm refused the write with a 404 on PUT. The packages exist and are owned',
          'by this account, so this is not a missing name: npm masks "no write access"',
          'as 404. NODE_AUTH_TOKEN is expired, revoked, or read-only.',
          '',
          'Rotate it: npmjs.com -> Access Tokens -> Generate (Automation, or Granular',
          'with read+write on openai-oxide and every openai-oxide-<platform> package),',
          'then update the NPM_TOKEN repository secret and re-run this workflow.',
          '',
        ].join('\n'),
      )
    }

    if (result.error) {
      console.error(result.error.message)
    }

    process.exit(result.status ?? 1)
  }
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})

function isAlreadyPublishedError(output) {
  return (
    /previously published versions?/i.test(output) ||
    /cannot publish over (?:the )?previously published versions?/i.test(output) ||
    /cannot publish over existing version/i.test(output) ||
    /You cannot publish over the previously published versions/i.test(output)
  )
}

// npm answers an unauthorised publish with 404 rather than 403, so a PUT 404 on
// a package that demonstrably exists means the token, not the name.
function isAuthError(output) {
  return (
    /404 Not Found - PUT/i.test(output) ||
    /(ENEEDAUTH|EAUTHUNKNOWN|E401|401 Unauthorized)/i.test(output) ||
    /you must be logged in to publish packages/i.test(output)
  )
}

function isRateLimitError(output) {
  return (
    /(Too Many Requests|rate limited|rate limit exceeded)/i.test(output) ||
    /status code 429/i.test(output)
  )
}

function parsePositiveInt(value, fallback) {
  if (value == null || value === '') {
    return fallback
  }

  const parsed = Number.parseInt(value, 10)
  if (Number.isFinite(parsed) && parsed > 0) {
    return parsed
  }

  return fallback
}
