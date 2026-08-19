const compiler = require("../relay-generic/compiler");
const RelayDB = require("../relay-generic/v6-reader");

module.exports = {
  RelayDB,
  compiler,
  compile: compiler.compile,
  verifyArtifact: compiler.verifyArtifact,
};
