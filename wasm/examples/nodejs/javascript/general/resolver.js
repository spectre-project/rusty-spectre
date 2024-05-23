// @ts-ignore
globalThis.WebSocket = require('websocket').w3cwebsocket; // W3C WebSocket module shim

const spectre = require('../../../../nodejs/spectre');
const { parseArgs } = require("../utils");
const {
    Resolver,
    Encoding,
    RpcClient,
} = spectre;

spectre.initConsolePanicHook();

const {
    networkId,
    encoding,
} = parseArgs();

(async () => {

    const resolver = new Resolver();
    // console.log(resolver);
    // process.exit(0);
    // let url = await resolver.getUrl(Encoding.Borsh, networkId);
    // console.log(url);
    const rpc = new RpcClient({
        // url,
        // encoding,
        resolver,
        networkId
    });

    // const rpc = await resolver.connect(networkId);
    await rpc.connect();
    console.log("Connected to", rpc.url);
    console.log("RPC", rpc);

    // console.log(`Connecting to ${rpc.url}`)

    const info = await rpc.getBlockDagInfo();
    console.log("GetBlockDagInfo response:", info);

    await rpc.disconnect();
})();
