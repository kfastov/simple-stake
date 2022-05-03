// TODO rewrite tests

import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { expect } from "chai";
import { SimpleStake } from "../target/types/simple_stake";

describe("simple-stake", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.SimpleStake as Program<SimpleStake>;

  it("Create pool!", async () => {
    const admin = program.provider.wallet;
    const mint = new anchor.web3.PublicKey("SQRNmMb9mKjjkihQS7fCmAwo3gVs1FSQBVeDZzA7CP3");

    console.log('Mint as decimal bytes:', [...mint.toBytes()]);

    const tokenProgramId = new anchor.web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    // pda (new authority)
    const [poolAuthority] = await anchor.web3.PublicKey
    .findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("pool-authority"),
      ],
      program.programId
    );

    const [tokenPool] = await anchor.web3.PublicKey
    .findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("pool-tokens"),
      ],
      program.programId
    );

    console.log('new tokenPool address:', tokenPool.toString());

    // Execute the RPC.
    await program.methods
      .createTokenPool()
      .accounts({
        admin: admin.publicKey,
        poolTokens: tokenPool,
        poolAuthority: poolAuthority,
        tokenProgram: tokenProgramId,
        mint,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([])
      .rpc();

    // let poolAccountState = await program.account.rewardSource.fetch(rewardSource.publicKey);
    // TODO poolAccountState = await Smth.TokenAccount.fetch(poolAccount.publicKey);
    // expect(rewardSourceState.admin).to.equal(admin.publicKey);
    // expect(rewardSourceState.isActive).to.equal(true);
    // TODO etc
  });

  it("Do stake!", async () => {

    const user = new anchor.web3.Keypair();
    const userToken = new anchor.web3.PublicKey("7vt63GE4hp7pB6RZjTX5aNWBQeEaBSV8uv8wuVGWt93g");
    const tokenProgramId = new anchor.web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

    console.log('Funding user wallet', user.publicKey.toString());
    // await program.provider.connection.requestAirdrop(user.publicKey, 1231920);
    await program.provider.connection.requestAirdrop(user.publicKey, 10000000);

    const [stakeInfo] = await anchor.web3.PublicKey
    .findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("stake-info"),
        user.publicKey.toBytes(),
      ],
      program.programId
    );

    const [tokenPool] = await anchor.web3.PublicKey
    .findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("pool-tokens"),
      ],
      program.programId
    );

    console.log('new tokenPool address:', tokenPool.toString());

    // Execute the RPC.
    await program.methods
      .stakeTokens()
      .accounts({
        user: user.publicKey,
        info: stakeInfo,
        userTokens: userToken,
        poolTokens: tokenPool,
        tokenProgram: tokenProgramId,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([user])
      .rpc({skipPreflight: true});

  });

});
