# Ephemeral Orderbook

User controlled account are sent to an ephemeral rollup to enable trades at the speed of magic.

## Usage

```bash
anchor build
anchor deploy
anchor test --skip-deploy
```

Splitting commands is useful because there can be a latency between deploying and program availability on MagicBlock. Once deployed, on can just run the `anchor test --skip-deploy` command as many times as needed.