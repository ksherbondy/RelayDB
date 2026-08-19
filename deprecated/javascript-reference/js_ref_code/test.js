const path = require("path");
const { execFileSync } = require("child_process");

const root = path.resolve(__dirname, "../..");
const testPath = path.join(root, "scripts", "relay-generic", "v6-tests.js");

execFileSync(process.execPath, [testPath], {
  cwd: root,
  stdio: "inherit",
});
