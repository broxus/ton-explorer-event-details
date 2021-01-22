<div align="center">
  <h1>TON Event Details</h1>
  <strong>WASM library to extract event details on the client</strong>
</div>

## Usage

```shell
npm install --save ton-explorer-event-details
```

## Example

```js
import * as addon from 'ton-explorer-event-details';

const TON_EVENT = '...base64 encoded account state...';

const ETH_ABI = `{
    "name": "TONStateChange",
    "inputs": [{"name":"state","type":"uint256"}], 
    "outputs": [] 
}`;

try {
    const details = addon.getDetails(TON_EVENT);
    console.log(details);

    const payload = addon.encodePayload(details, ETH_ABI);
    console.log(payload);
} catch (e) {
    console.error(e);
}
```
