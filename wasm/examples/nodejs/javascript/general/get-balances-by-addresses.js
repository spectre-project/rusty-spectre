// @ts-ignore
globalThis.WebSocket = require('websocket').w3cwebsocket; // W3C WebSocket module shim

const spectre = require('../../../../nodejs/spectre');
const { parseArgs } = require("../utils");
const {
    RpcClient,
    Resolver,
} = spectre;

spectre.initConsolePanicHook();

const {
    networkId,
    encoding,
} = parseArgs();

(async () => {

    const rpc = new RpcClient({
        // url : "127.0.0.1",
        // encoding,
        resolver: new Resolver(),
        networkId : "mainnet"
    });
    console.log(`Resolving RPC endpoint...`);
    await rpc.connect();
    console.log(`Connecting to ${rpc.url}`)

    const info = await rpc.getBalancesByAddresses({ addresses : ["spectre:qpamkvhgh0kzx50gwvvp5xs8ktmqutcy3dfs9dc3w7lm9rq0zs76vf959mmrp"]});
    // const info = await rpc.getBalancesByAddresses(["spectre:qpamkvhgh0kzx50gwvvp5xs8ktmqutcy3dfs9dc3w7lm9rq0zs76vf959mmrp"]);
    console.log("GetBalancesByAddresses response:", info);

    await rpc.disconnect();
    console.log("bye!");
})();
