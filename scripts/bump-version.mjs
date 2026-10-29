// Patch-bumps the app version (x.y.z -> x.y.z+1) in every file that carries it.
import { readFileSync, writeFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

const root = join(dirname(fileURLToPath(import.meta.url)), "..")
const file = (p) => join(root, p)

const pkgPath = file("package.json")
const current = JSON.parse(readFileSync(pkgPath, "utf8")).version
const m = /^(\d+)\.(\d+)\.(\d+)$/.exec(current)
if (!m) throw new Error(`Unsupported version "${current}"`)
const next = `${m[1]}.${m[2]}.${Number(m[3]) + 1}`

// [path, regex; version sits between groups 1 and 2]
const edits = [
  ["package.json", /^(\s*"version":\s*")[^"]+(")/m],
  ["src-tauri/tauri.conf.json", /^(\s*"version":\s*")[^"]+(")/m],
  ["src-tauri/Cargo.toml", /^(version\s*=\s*")[^"]+(")/m],
  ["src-tauri/Cargo.lock", /(name = "easy-vial"\nversion = ")[^"]+(")/],
]

const tag = "[bump-version]"
console.log(`${tag} current version: ${current}`)
console.log(`${tag} next version:    ${next}`)

// Read + validate everything first so a failure never leaves files half-bumped.
const updated = edits.map(([p, re]) => {
  const src = readFileSync(file(p), "utf8")
  const found = re.exec(src)
  if (!found) throw new Error(`Version not found in ${p}`)
  const old = /"?([\d.]+)"?$/.exec(found[0])?.[1]
  if (old !== current) console.warn(`${tag} WARN ${p} was at ${old}, expected ${current} (files out of sync)`)
  return [p, old, src.replace(re, `$1${next}$2`)]
})
for (const [p, old, out] of updated) {
  writeFileSync(file(p), out)
  console.log(`${tag}   ${p}: ${old} -> ${next}`)
}

console.log(`${tag} done: ${current} -> ${next}`)
