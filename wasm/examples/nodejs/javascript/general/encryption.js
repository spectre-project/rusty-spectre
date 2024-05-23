const spectre = require('../../../../nodejs/spectre');

spectre.initConsolePanicHook();

(async () => {

    let encrypted = spectre.encryptXChaCha20Poly1305("my message", "my_password");
    console.log("encrypted:", encrypted);
    let decrypted = spectre.decryptXChaCha20Poly1305(encrypted, "my_password");
    console.log("decrypted:", decrypted);

})();
