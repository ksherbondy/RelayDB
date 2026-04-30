import RelayDB from "../../packages/relaydb-js/src/index.js";

(async () => {
  const db = await RelayDB.open("../../builds/bacon_standard.relay");

  console.log("Anchors:", db.anchors());

  const bacon = db.get("kevin_bacon");
  console.log("Node:", bacon);

  const result = db.jump("the_terminal", { filter: "Drama" });
  console.log("Traversal:", result);

  console.log("Integrity:", db.verifyIntegrity());
})();