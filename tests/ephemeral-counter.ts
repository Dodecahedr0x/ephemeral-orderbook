import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EphemeralOrderbook } from "../target/types/ephemeral_orderbook";
import {
  DELEGATION_PROGRAM_ID,
  GetCommitmentSignature,
} from "@magicblock-labs/ephemeral-rollups-sdk";
import token, {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountIdempotent,
  createMint,
  getAssociatedTokenAddressSync,
  mintToChecked,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import takerKpBytes from "./taker.json";
import * as dotenv from "dotenv";
import { assert } from "chai";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
dotenv.config();

const ORDERBOOK_PDA_SEED = "orderbook:"; // 5RgeA5P8bRaynJovch3zQURfJxXL3QK2JYg1YamSvyLb

describe("ephemeral-orderbook", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  console.log(process.env.PROVIDER_ENDPOINT);
  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      process.env.PROVIDER_ENDPOINT || "https://devnet.magicblock.app/",
      {
        wsEndpoint: process.env.WS_ENDPOINT || "wss://devnet.magicblock.app/",
      }
    ),
    anchor.Wallet.local()
  );

  const program = anchor.workspace
    .EphemeralOrderbook as Program<EphemeralOrderbook>;
  const ephemeralProgram = new Program(program.idl, providerEphemeralRollup);
  const orderbookId = anchor.web3.Keypair.generate().publicKey;
  const [orderbookPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(ORDERBOOK_PDA_SEED), orderbookId.toBuffer()],
    program.programId
  );
  const quantity = new anchor.BN(10000000);
  const takerKp = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(takerKpBytes)
  );
  let baseMint: anchor.web3.PublicKey;
  let quoteMint: anchor.web3.PublicKey;
  let makerBaseAta: anchor.web3.PublicKey;
  let makerQuoteAta: anchor.web3.PublicKey;
  let takerBaseAta: anchor.web3.PublicKey;
  let takerQuoteAta: anchor.web3.PublicKey;
  let orderbookBaseAta: anchor.web3.PublicKey;
  let orderbookQuoteAta: anchor.web3.PublicKey;

  before(async () => {
    baseMint = await createMint(
      provider.connection,
      anchor.Wallet.local().payer,
      provider.wallet.publicKey,
      provider.wallet.publicKey,
      6
    );
    quoteMint = await createMint(
      provider.connection,
      anchor.Wallet.local().payer,
      provider.wallet.publicKey,
      provider.wallet.publicKey,
      6
    );

    makerBaseAta = await createAssociatedTokenAccountIdempotent(
      provider.connection,
      anchor.Wallet.local().payer,
      baseMint,
      provider.wallet.publicKey
    );
    makerQuoteAta = await createAssociatedTokenAccountIdempotent(
      provider.connection,
      anchor.Wallet.local().payer,
      quoteMint,
      provider.wallet.publicKey
    );
    takerBaseAta = await createAssociatedTokenAccountIdempotent(
      provider.connection,
      anchor.Wallet.local().payer,
      baseMint,
      takerKp.publicKey
    );
    takerQuoteAta = await createAssociatedTokenAccountIdempotent(
      provider.connection,
      anchor.Wallet.local().payer,
      quoteMint,
      takerKp.publicKey
    );
    orderbookBaseAta = await createAssociatedTokenAccountIdempotent(
      provider.connection,
      anchor.Wallet.local().payer,
      baseMint,
      orderbookPda,
      undefined,
      undefined,
      undefined,
      true
    );
    orderbookQuoteAta = await createAssociatedTokenAccountIdempotent(
      provider.connection,
      anchor.Wallet.local().payer,
      quoteMint,
      orderbookPda,
      undefined,
      undefined,
      undefined,
      true
    );

    await mintToChecked(
      provider.connection,
      anchor.Wallet.local().payer,
      baseMint,
      makerBaseAta,
      anchor.Wallet.local().payer,
      quantity.toNumber(),
      6
    );
    await mintToChecked(
      provider.connection,
      anchor.Wallet.local().payer,
      quoteMint,
      takerQuoteAta,
      anchor.Wallet.local().payer,
      quantity.toNumber(),
      6
    );
  });

  it("Initializes the orderbook.", async () => {
    await program.methods
      .initializeOrderbook({
        id: orderbookId,
        baseMint: baseMint,
        quoteMint: quoteMint,
      })
      .accountsPartial({
        orderbook: orderbookPda,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    const orderbookAccount = await program.account.orderbook.fetch(
      orderbookPda
    );
    assert.equal(orderbookAccount.id.toBase58(), orderbookId.toBase58());
    assert.equal(orderbookAccount.baseMint.toBase58(), baseMint.toBase58());
    assert.equal(orderbookAccount.quoteMint.toBase58(), quoteMint.toBase58());
    assert.equal(orderbookAccount.userBalances.length, 0);
    assert.equal(orderbookAccount.orders.length, 0);
  });

  it("Create users", async () => {
    await program.methods
      .createUser()
      .accountsPartial({ orderbook: orderbookPda })
      .rpc();
    await program.methods
      .createUser()
      .accountsPartial({ orderbook: orderbookPda, user: takerKp.publicKey })
      .signers([takerKp])
      .rpc();

    const orderbookAccount = await program.account.orderbook.fetch(
      orderbookPda
    );
    assert.equal(orderbookAccount.id.toBase58(), orderbookId.toBase58());
    assert.equal(orderbookAccount.baseMint.toBase58(), baseMint.toBase58());
    assert.equal(orderbookAccount.quoteMint.toBase58(), quoteMint.toBase58());
    assert.equal(orderbookAccount.orders.length, 0);
    assert.equal(orderbookAccount.userBalances.length, 2);
    assert.equal(
      orderbookAccount.userBalances[0].user.toBase58(),
      provider.wallet.publicKey.toBase58()
    );
    assert.equal(
      orderbookAccount.userBalances[1].user.toBase58(),
      takerKp.publicKey.toBase58()
    );
  });

  it("Deposit tokens", async () => {
    await program.methods
      .deposit({ amount: quantity })
      .accountsPartial({
        orderbook: orderbookPda,
        mint: baseMint,
        userTokenAccount: makerBaseAta,
        orderbookTokenAccount: orderbookBaseAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });
    await program.methods
      .deposit({ amount: quantity })
      .accountsPartial({
        user: takerKp.publicKey,
        orderbook: orderbookPda,
        mint: quoteMint,
        userTokenAccount: takerQuoteAta,
        orderbookTokenAccount: orderbookQuoteAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([takerKp, anchor.Wallet.local().payer])
      .rpc();

    const orderbookAccount = await program.account.orderbook.fetch(
      orderbookPda
    );
    assert.equal(
      orderbookAccount.userBalances[0].baseBalance.toString(),
      quantity.toString()
    );
    assert.equal(orderbookAccount.userBalances[0].quoteBalance.toString(), "0");
    assert.equal(orderbookAccount.userBalances[1].baseBalance.toString(), "0");
    assert.equal(
      orderbookAccount.userBalances[1].quoteBalance.toString(),
      quantity.toString()
    );
  });

  it("Delegate the orderbook", async () => {
    let tx = await program.methods
      .delegateOrderbook({ id: orderbookId })
      .accountsPartial({
        payer: provider.wallet.publicKey,
        pda: orderbookPda,
      })
      .transaction();
    tx.feePayer = provider.wallet.publicKey;
    tx.recentBlockhash = (
      await provider.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    await provider.sendAndConfirm(tx, [], {
      skipPreflight: true,
      commitment: "confirmed",
    });

    const account = await provider.connection.getAccountInfo(orderbookPda);
    assert.equal(account.owner.toBase58(), DELEGATION_PROGRAM_ID.toBase58());
  });

  it("Create orders", async () => {
    let makerTx = await program.methods
      .createOrder({
        order: {
          owner: provider.wallet.publicKey,
          matchTimestamp: null,
          orderType: { sell: {} },
          price: new anchor.BN(1),
          quantity,
        },
      })
      .accountsPartial({
        user: provider.wallet.publicKey,
        orderbook: orderbookPda,
      })
      .transaction();
    let takerTx = await program.methods
      .createOrder({
        order: {
          owner: takerKp.publicKey,
          matchTimestamp: null,
          orderType: { buy: {} },
          price: new anchor.BN(1),
          quantity,
        },
      })
      .accountsPartial({
        user: takerKp.publicKey,
        orderbook: orderbookPda,
      })
      .signers([])
      .transaction();

    for (let { tx, wallet, signers } of [
      {
        tx: makerTx,
        wallet: providerEphemeralRollup.wallet,
      },
      { tx: takerTx, wallet: new anchor.Wallet(takerKp), signers: [takerKp] },
    ]) {
      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (
        await providerEphemeralRollup.connection.getLatestBlockhash()
      ).blockhash;
      tx = await wallet.signTransaction(tx);
      await new anchor.AnchorProvider(
        providerEphemeralRollup.connection,
        wallet
      ).sendAndConfirm(tx, signers);
    }

    const orderbookAccount = await ephemeralProgram.account.orderbook.fetch(
      orderbookPda
    );
    assert.equal(orderbookAccount.orders.length, 2);
    assert.equal(
      orderbookAccount.orders[0].owner.toBase58(),
      provider.wallet.publicKey.toBase58()
    );
    assert.equal(
      orderbookAccount.orders[1].owner.toBase58(),
      takerKp.publicKey.toBase58()
    );
  });

  it("Match orders", async () => {
    const oracleData = {
      symbol: "SOL/USD",
      id: anchor.web3.PublicKey.unique(),
      temporalNumericValue: {
        timestampNs: new anchor.BN(1234567890),
        quantizedValue: new anchor.BN(1),
      },
      publisherMerkleRoot: Array.from(anchor.web3.PublicKey.unique().toBytes()),
      valueComputeAlgHash: Array.from(anchor.web3.PublicKey.unique().toBytes()),
      r: Array.from(anchor.web3.PublicKey.unique().toBytes()),
      s: Array.from(anchor.web3.PublicKey.unique().toBytes()),
      v: new anchor.BN(1),
    };
    let tx = await program.methods
      .matchOrder({
        oracleData,
        maker: provider.wallet.publicKey,
        taker: takerKp.publicKey,
        makerIndex: new anchor.BN(0),
        takerIndex: new anchor.BN(1),
      })
      .accountsPartial({
        orderbook: orderbookPda,
      })
      .transaction();
    tx.feePayer = providerEphemeralRollup.wallet.publicKey;
    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);

    await providerEphemeralRollup.sendAndConfirm(tx);

    const orderbookAccount = await ephemeralProgram.account.orderbook.fetch(
      orderbookPda
    );
    assert.equal(orderbookAccount.orders.length, 0);
  });

  // it("Undelegate the orderbook and withdraw tokens", async () => {
  //   let tx = await program.methods
  //     .incrementAndUndelegate()
  //     .accounts({
  //       payer: providerEphemeralRollup.wallet.publicKey,
  //       // @ts-ignore
  //       counter: pda,
  //     })
  //     .transaction();
  //   tx.feePayer = provider.wallet.publicKey;
  //   tx.recentBlockhash = (
  //     await providerEphemeralRollup.connection.getLatestBlockhash()
  //   ).blockhash;
  //   tx = await providerEphemeralRollup.wallet.signTransaction(tx);

  //   const txSign = await providerEphemeralRollup.sendAndConfirm(tx);
  //   console.log("Increment Tx and Commit: ", txSign);

  //   const counterAccount = await ephemeralProgram.account.counter.fetch(pda);
  //   console.log("Counter: ", counterAccount.count.toString());

  //   // Await for the commitment on the base layer
  //   const txCommitSgn = await GetCommitmentSignature(
  //     txSign,
  //     providerEphemeralRollup.connection
  //   );
  //   console.log("Account commit signature:", txCommitSgn);
  //   const latestBlockhash = await provider.connection.getLatestBlockhash();
  //   await provider.connection.confirmTransaction(
  //     {
  //       signature: txCommitSgn,
  //       ...latestBlockhash,
  //     },
  //     "confirmed"
  //   );

  //   const counterAccount = await program.account.counter.fetch(pda);
  //   console.log("Counter: ", counterAccount.count.toString());
  // });
});
