import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EphemeralOrderbook } from "../target/types/ephemeral_orderbook";
import {
  DELEGATION_PROGRAM_ID,
  GetCommitmentSignature,
} from "@magicblock-labs/ephemeral-rollups-sdk";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotent,
  createMint,
  mintToChecked,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import takerKpBytes from "./taker.json";
import * as dotenv from "dotenv";
import { assert } from "chai";
dotenv.config();

const ORDERBOOK_PDA_SEED = "orderbook:";
const TRADER_PDA_SEED = "trader:";

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
  const takerKp = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(takerKpBytes)
  );
  const orderbookId = anchor.web3.Keypair.generate().publicKey;
  const [orderbookPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(ORDERBOOK_PDA_SEED), orderbookId.toBuffer()],
    program.programId
  );
  const [makerPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(TRADER_PDA_SEED),
      orderbookPda.toBuffer(),
      provider.wallet.publicKey.toBuffer(),
    ],
    program.programId
  );
  const [takerPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(TRADER_PDA_SEED),
      orderbookPda.toBuffer(),
      takerKp.publicKey.toBuffer(),
    ],
    program.programId
  );
  const quantity = new anchor.BN(10000000);
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
  });

  it("Create traders", async () => {
    await program.methods
      .createTrader()
      .accountsPartial({
        orderbook: orderbookPda,
        trader: makerPda,
        user: provider.wallet.publicKey,
      })
      .rpc();
    await program.methods
      .createTrader()
      .accountsPartial({
        orderbook: orderbookPda,
        trader: takerPda,
        user: takerKp.publicKey,
      })
      .signers([takerKp])
      .rpc();

    const makerAccount = await program.account.trader.fetch(makerPda);
    assert.equal(
      makerAccount.user.toBase58(),
      provider.wallet.publicKey.toBase58()
    );
    assert.equal(makerAccount.orderbook.toBase58(), orderbookPda.toBase58());
    assert.equal(makerAccount.orders.length, 0);
    assert.equal(makerAccount.baseBalance.toString(), "0");
    assert.equal(makerAccount.quoteBalance.toString(), "0");

    const takerAccount = await program.account.trader.fetch(takerPda);
    assert.equal(takerAccount.user.toBase58(), takerKp.publicKey.toBase58());
    assert.equal(takerAccount.orderbook.toBase58(), orderbookPda.toBase58());
    assert.equal(takerAccount.orders.length, 0);
    assert.equal(takerAccount.baseBalance.toString(), "0");
    assert.equal(takerAccount.quoteBalance.toString(), "0");
  });

  it("Deposit tokens", async () => {
    await program.methods
      .deposit({ amount: quantity })
      .accountsPartial({
        orderbook: orderbookPda,
        trader: makerPda,
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
        trader: takerPda,
        mint: quoteMint,
        userTokenAccount: takerQuoteAta,
        orderbookTokenAccount: orderbookQuoteAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([takerKp, anchor.Wallet.local().payer])
      .rpc();

    const makerAccount = await program.account.trader.fetch(makerPda);
    assert.equal(makerAccount.orders.length, 0);
    assert.equal(makerAccount.baseBalance.toString(), quantity.toString());
    assert.equal(makerAccount.quoteBalance.toString(), "0");

    const takerAccount = await program.account.trader.fetch(takerPda);
    assert.equal(takerAccount.orders.length, 0);
    assert.equal(takerAccount.baseBalance.toString(), "0");
    assert.equal(takerAccount.quoteBalance.toString(), quantity.toString());
  });

  it("Delegate traders", async () => {
    let makerTx = await program.methods
      .delegateTrader({ orderbook: orderbookPda })
      .accountsPartial({
        payer: provider.wallet.publicKey,
        pda: makerPda,
      })
      .transaction();
    makerTx.feePayer = provider.wallet.publicKey;
    makerTx.recentBlockhash = (
      await provider.connection.getLatestBlockhash()
    ).blockhash;
    makerTx = await providerEphemeralRollup.wallet.signTransaction(makerTx);
    await provider.sendAndConfirm(makerTx, [], {
      skipPreflight: true,
      commitment: "confirmed",
    });

    let takerTx = await program.methods
      .delegateTrader({ orderbook: orderbookPda })
      .accountsPartial({
        payer: takerKp.publicKey,
        pda: takerPda,
      })
      .signers([takerKp])
      .transaction();
    takerTx.feePayer = takerKp.publicKey;
    takerTx.recentBlockhash = (
      await provider.connection.getLatestBlockhash()
    ).blockhash;
    takerTx = await new anchor.Wallet(takerKp).signTransaction(takerTx);
    const sig = await provider.connection.sendRawTransaction(
      takerTx.serialize()
    );
    await provider.connection.confirmTransaction(sig);

    const makerAccount = await provider.connection.getAccountInfo(makerPda);
    assert.equal(
      makerAccount.owner.toBase58(),
      DELEGATION_PROGRAM_ID.toBase58()
    );
    const takerAccount = await provider.connection.getAccountInfo(takerPda);
    assert.equal(
      takerAccount.owner.toBase58(),
      DELEGATION_PROGRAM_ID.toBase58()
    );
  });

  it("Create orders", async () => {
    let makerTx = await program.methods
      .createOrder({
        order: {
          matchTimestamp: null,
          orderType: { sell: {} },
          price: new anchor.BN(1),
          quantity,
        },
      })
      .accountsPartial({
        user: provider.wallet.publicKey,
        orderbook: orderbookPda,
        trader: makerPda,
      })
      .transaction();
    let takerTx = await program.methods
      .createOrder({
        order: {
          matchTimestamp: null,
          orderType: { buy: {} },
          price: new anchor.BN(1),
          quantity,
        },
      })
      .accountsPartial({
        user: takerKp.publicKey,
        orderbook: orderbookPda,
        trader: takerPda,
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

    const makerAccount = await ephemeralProgram.account.trader.fetch(makerPda);
    assert.equal(makerAccount.orders.length, 1);
    assert.equal(makerAccount.baseBalance.toString(), "0");
    assert.equal(makerAccount.quoteBalance.toString(), "0");

    const takerAccount = await ephemeralProgram.account.trader.fetch(takerPda);
    assert.equal(takerAccount.orders.length, 1);
    assert.equal(takerAccount.baseBalance.toString(), "0");
    assert.equal(takerAccount.quoteBalance.toString(), "0");
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
        takerIndex: new anchor.BN(0),
      })
      .accountsPartial({
        orderbook: orderbookPda,
        maker: makerPda,
        taker: takerPda,
      })
      .transaction();
    tx.feePayer = providerEphemeralRollup.wallet.publicKey;
    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);

    await providerEphemeralRollup.sendAndConfirm(tx);

    const makerAccount = await ephemeralProgram.account.trader.fetch(makerPda);
    assert.equal(makerAccount.orders.length, 0);
    assert.equal(makerAccount.baseBalance.toString(), "0");
    assert.equal(makerAccount.quoteBalance.toString(), quantity.toString());

    const takerAccount = await ephemeralProgram.account.trader.fetch(takerPda);
    assert.equal(takerAccount.orders.length, 0);
    assert.equal(takerAccount.baseBalance.toString(), quantity.toString());
    assert.equal(takerAccount.quoteBalance.toString(), "0");
  });

  it("Undelegate traders", async () => {
    let makerTx = await program.methods
      .undelegateTrader()
      .accountsPartial({
        payer: providerEphemeralRollup.wallet.publicKey,
        trader: makerPda,
      })
      .transaction();
    makerTx.feePayer = provider.wallet.publicKey;
    makerTx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    makerTx = await providerEphemeralRollup.wallet.signTransaction(makerTx);
    const makerTxSign = await providerEphemeralRollup.sendAndConfirm(makerTx);

    let takerTx = await program.methods
      .undelegateTrader()
      .accountsPartial({
        payer: takerKp.publicKey,
        trader: takerPda,
      })
      .transaction();
    takerTx.feePayer = takerKp.publicKey;
    takerTx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    takerTx = await new anchor.Wallet(takerKp).signTransaction(takerTx);
    const takerTxSign =
      await providerEphemeralRollup.connection.sendRawTransaction(
        takerTx.serialize()
      );
    await providerEphemeralRollup.connection.confirmTransaction(takerTxSign);

    // Await for the commitment on the base layer
    await Promise.all([
      await GetCommitmentSignature(
        makerTxSign,
        providerEphemeralRollup.connection
      ),
      await GetCommitmentSignature(
        takerTxSign,
        providerEphemeralRollup.connection
      ),
    ]);
  });

  it("Withdraw tokens on the base layer", async () => {
    await program.methods
      .withdraw({ amount: quantity })
      .accountsPartial({
        user: provider.wallet.publicKey,
        orderbook: orderbookPda,
        trader: makerPda,
        mint: quoteMint,
        userTokenAccount: makerQuoteAta,
        orderbookTokenAccount: orderbookQuoteAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    await program.methods
      .withdraw({ amount: quantity })
      .accountsPartial({
        user: takerKp.publicKey,
        orderbook: orderbookPda,
        trader: takerPda,
        mint: baseMint,
        userTokenAccount: takerBaseAta,
        orderbookTokenAccount: orderbookBaseAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([takerKp])
      .rpc();

    const makerAccount = await provider.connection.getTokenAccountBalance(
      makerQuoteAta
    );
    assert.equal(makerAccount.value.amount.toString(), quantity.toString());

    const takerAccount = await provider.connection.getTokenAccountBalance(
      takerBaseAta
    );
    assert.equal(takerAccount.value.amount.toString(), quantity.toString());
  });
});
