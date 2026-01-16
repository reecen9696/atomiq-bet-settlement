import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";
import { 
  PublicKey, 
  SystemProgram, 
  LAMPORTS_PER_SOL,
  Keypair 
} from "@solana/web3.js";
import { expect } from "chai";

describe("vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Vault as Program<Vault>;
  const authority = provider.wallet as anchor.Wallet;
  const user = Keypair.generate();
  
  let casinoPda: PublicKey;
  let casinoBump: number;
  let vaultPda: PublicKey;
  let vaultBump: number;
  let vaultAuthorityPda: PublicKey;
  let rateLimiterPda: PublicKey;

  before(async () => {
    // Airdrop SOL to test user
    const airdropSig = await provider.connection.requestAirdrop(
      user.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    // Derive PDAs
    [casinoPda, casinoBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("casino")],
      program.programId
    );

    [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), casinoPda.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );

    [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault-authority"), casinoPda.toBuffer()],
      program.programId
    );

    [rateLimiterPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("rate-limiter"), user.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("Casino Initialization", () => {
    it("Initializes casino vault", async () => {
      await program.methods
        .initializeCasinoVault(authority.publicKey)
        .accounts({
          casino: casinoPda,
          vaultAuthority: vaultAuthorityPda,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const casinoAccount = await program.account.casino.fetch(casinoPda);
      expect(casinoAccount.authority.toString()).to.equal(
        authority.publicKey.toString()
      );
      expect(casinoAccount.paused).to.be.false;
      expect(casinoAccount.totalBets.toNumber()).to.equal(0);
    });

    it("Cannot initialize casino twice", async () => {
      try {
        await program.methods
          .initializeCasinoVault(authority.publicKey)
          .accounts({
            casino: casinoPda,
            vaultAuthority: vaultAuthorityPda,
            authority: authority.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("already in use");
      }
    });
  });

  describe("User Vault", () => {
    it("Initializes user vault", async () => {
      await program.methods
        .initializeVault()
        .accounts({
          vault: vaultPda,
          casino: casinoPda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const vaultAccount = await program.account.vault.fetch(vaultPda);
      expect(vaultAccount.owner.toString()).to.equal(user.publicKey.toString());
      expect(vaultAccount.casino.toString()).to.equal(casinoPda.toString());
      expect(vaultAccount.solBalance.toNumber()).to.equal(0);
    });

    it("Cannot initialize vault twice", async () => {
      try {
        await program.methods
          .initializeVault()
          .accounts({
            vault: vaultPda,
            casino: casinoPda,
            user: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("already in use");
      }
    });
  });

  describe("Deposits", () => {
    it("Deposits SOL to vault", async () => {
      const depositAmount = 1 * LAMPORTS_PER_SOL;

      await program.methods
        .depositSol(new anchor.BN(depositAmount))
        .accounts({
          vault: vaultPda,
          casino: casinoPda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const vaultAccount = await program.account.vault.fetch(vaultPda);
      expect(vaultAccount.solBalance.toNumber()).to.equal(depositAmount);
    });

    it("Deposits additional SOL", async () => {
      const depositAmount = 0.5 * LAMPORTS_PER_SOL;

      await program.methods
        .depositSol(new anchor.BN(depositAmount))
        .accounts({
          vault: vaultPda,
          casino: casinoPda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const vaultAccount = await program.account.vault.fetch(vaultPda);
      expect(vaultAccount.solBalance.toNumber()).to.equal(
        1.5 * LAMPORTS_PER_SOL
      );
    });
  });

  describe("Allowances", () => {
    let allowancePda: PublicKey;
    const allowanceAmount = 0.5 * LAMPORTS_PER_SOL;
    const duration = 3600; // 1 hour

    it("Approves spending allowance", async () => {
      const timestamp = Math.floor(Date.now() / 1000);
      
      [allowancePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("allowance"),
          user.publicKey.toBuffer(),
          casinoPda.toBuffer(),
          Buffer.from(new anchor.BN(timestamp).toArray("le", 8)),
        ],
        program.programId
      );

      await program.methods
        .approveAllowance(
          new anchor.BN(allowanceAmount),
          new anchor.BN(duration),
          SystemProgram.programId // SOL token mint (System Program for native SOL)
        )
        .accounts({
          vault: vaultPda,
          casino: casinoPda,
          allowance: allowancePda,
          rateLimiter: rateLimiterPda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const allowanceAccount = await program.account.allowance.fetch(
        allowancePda
      );
      expect(allowanceAccount.amount.toNumber()).to.equal(allowanceAmount);
      expect(allowanceAccount.spent.toNumber()).to.equal(0);
      expect(allowanceAccount.revoked).to.be.false;
    });

    it("Cannot approve allowance exceeding maximum duration", async () => {
      const timestamp = Math.floor(Date.now() / 1000) + 1;
      const [invalidAllowancePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("allowance"),
          user.publicKey.toBuffer(),
          casinoPda.toBuffer(),
          Buffer.from(new anchor.BN(timestamp).toArray("le", 8)),
        ],
        program.programId
      );

      try {
        await program.methods
          .approveAllowance(
            new anchor.BN(allowanceAmount),
            new anchor.BN(86401), // > 24 hours
            SystemProgram.programId
          )
          .accounts({
            vault: vaultPda,
            casino: casinoPda,
            allowance: invalidAllowancePda,
            rateLimiter: rateLimiterPda,
            user: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("AllowanceDurationTooLong");
      }
    });

    it("User can revoke allowance", async () => {
      await program.methods
        .revokeAllowance()
        .accounts({
          allowance: allowancePda,
          user: user.publicKey,
        })
        .signers([user])
        .rpc();

      const allowanceAccount = await program.account.allowance.fetch(
        allowancePda
      );
      expect(allowanceAccount.revoked).to.be.true;
    });
  });

  describe("Withdrawals", () => {
    it("User can withdraw SOL", async () => {
      const withdrawAmount = 0.5 * LAMPORTS_PER_SOL;
      const vaultBefore = await program.account.vault.fetch(vaultPda);

      await program.methods
        .withdrawSol(new anchor.BN(withdrawAmount))
        .accounts({
          vault: vaultPda,
          casino: casinoPda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const vaultAfter = await program.account.vault.fetch(vaultPda);
      expect(vaultAfter.solBalance.toNumber()).to.equal(
        vaultBefore.solBalance.toNumber() - withdrawAmount
      );
    });

    it("Cannot withdraw more than balance", async () => {
      const vaultAccount = await program.account.vault.fetch(vaultPda);
      const excessAmount = vaultAccount.solBalance.toNumber() + 1;

      try {
        await program.methods
          .withdrawSol(new anchor.BN(excessAmount))
          .accounts({
            vault: vaultPda,
            casino: casinoPda,
            user: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("InsufficientBalance");
      }
    });
  });

  describe("Emergency Pause", () => {
    it("Authority can pause casino", async () => {
      await program.methods
        .pauseCasino()
        .accounts({
          casino: casinoPda,
          authority: authority.publicKey,
        })
        .rpc();

      const casinoAccount = await program.account.casino.fetch(casinoPda);
      expect(casinoAccount.paused).to.be.true;
    });

    it("Cannot approve allowance when paused", async () => {
      const timestamp = Math.floor(Date.now() / 1000) + 2;
      const [allowancePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("allowance"),
          user.publicKey.toBuffer(),
          casinoPda.toBuffer(),
          Buffer.from(new anchor.BN(timestamp).toArray("le", 8)),
        ],
        program.programId
      );

      try {
        await program.methods
          .approveAllowance(
            new anchor.BN(0.1 * LAMPORTS_PER_SOL),
            new anchor.BN(3600),
            SystemProgram.programId
          )
          .accounts({
            vault: vaultPda,
            casino: casinoPda,
            allowance: allowancePda,
            rateLimiter: rateLimiterPda,
            user: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("CasinoPaused");
      }
    });

    it("Authority can unpause casino", async () => {
      await program.methods
        .unpauseCasino()
        .accounts({
          casino: casinoPda,
          authority: authority.publicKey,
        })
        .rpc();

      const casinoAccount = await program.account.casino.fetch(casinoPda);
      expect(casinoAccount.paused).to.be.false;
    });

    it("Non-authority cannot pause casino", async () => {
      try {
        await program.methods
          .pauseCasino()
          .accounts({
            casino: casinoPda,
            authority: user.publicKey,
          })
          .signers([user])
          .rpc();
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("UnauthorizedAuthority");
      }
    });
  });
});
