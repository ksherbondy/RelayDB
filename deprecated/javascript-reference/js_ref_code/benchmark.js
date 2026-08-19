const path = require("path");
const { execFileSync } = require("child_process");

const root = path.resolve(__dirname, "../..");
const benchmarkPath = path.join(root, "scripts", "relay-generic", "fair-stress-benchmark.js");
const source = process.argv[2] || "datasets/generated/merged/people-companies.10000x100000.4tag.merged.jsonl";
const artifact = process.argv[3] || "builds/generic-versions/v6-large.relay";
const queries = process.argv[4] || "1000";

execFileSync(process.execPath, [benchmarkPath, source, artifact, queries], {
  cwd: root,
  stdio: "inherit",
});
