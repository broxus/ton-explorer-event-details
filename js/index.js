const TON_EVENT = "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";

const ETH_ABI = `{ "name": "TONStateChange", "inputs": [ {"name":"state","type":"uint256"} ], "outputs": [ ] }`;

import("../pkg").then(module => {
    const details = module.get_details(TON_EVENT);
    console.log(details);

    const payload = module.encode_eth_payload(details, ETH_ABI);
    console.log(payload);
}).catch(console.error);
