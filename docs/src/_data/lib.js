import { readFileSync } from "node:fs";
import { resolve } from "node:path";

// Read the published library's version straight from the root package.json
// (one level above the docs workspace), so the docs always show the real
// shipped version. Resolve from cwd (the docs/ dir during the build) rather
// than import.meta.url, which 11ty's data loader rebases unpredictably.
const rootPkg = resolve(process.cwd(), "..", "package.json");
const { version } = JSON.parse(readFileSync(rootPkg, "utf8"));

export default { version };
