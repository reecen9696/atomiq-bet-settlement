# Blockchain Integration Guide

## Current Status: ⚠️ Simulated Transactions

The system currently uses **simulated** Solana transactions. Here's what needs to be done to enable real blockchain settlement:

---

## 🎯 What's Already Built

### ✅ Solana Vault Program (Anchor)
**Location:** `programs/vault/src/`

**Program ID:** `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`

**Instructions Implemented:**
1. `initialize_vault` - Create user PDA vault
2. `initialize_casino_vault` - Create casino vault (admin)
3. `deposit_sol` - Deposit SOL to vault
4. `deposit_spl` - Deposit USDC to vault
5. `approve_allowance` - One-time approval for gasless bets
6. `revoke_allowance` - Cancel allowance
7. `spend_from_allowance` - Processor spends without user signature ✨
8. `payout` - Casino pays winnings
9. `withdraw_sol` - User withdraws SOL
10. `withdraw_spl` - User withdraws USDC
11. `pause_casino` - Emergency pause (admin)

**Key Feature: Gasless Betting**
- User approves allowance once
- Processor can spend from allowance without user signature
- User maintains custody of funds in their vault (PDA)

### ✅ Frontend Components
**Location:** `apps/frontend/src/components/`

**Components:**
- `VaultDashboard.tsx` - Shows balance, allowance status
- `BetInterface.tsx` - Place bets UI
- `BetHistory.tsx` - View past bets
- `WalletConnect.tsx` - Privy wallet integration

**Status:** UI built but using mock data

### ✅ Backend API
**Endpoints:**
- `POST /api/bets` - Create bet (stores in DB)
- `GET /api/bets/:id` - Get bet details
- `GET /api/external/bets/pending` - Processor polls for bets

**Status:** Working with database

---

## 🚧 What's Missing for Blockchain Integration

### 1. Deploy Anchor Program

**Current:** Program code exists but not deployed  
**Need:**

```bash
# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Build program
cd programs/vault
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Update program ID in code if it changes
anchor keys list
```

**Update these files with deployed program ID:**
- `programs/vault/Anchor.toml`
- `programs/vault/src/lib.rs` (declare_id!)
- `services/backend/.env` (VAULT_PROGRAM_ID)
- `services/processor/.env` (VAULT_PROGRAM_ID)
- `apps/frontend/.env` (NEXT_PUBLIC_VAULT_PROGRAM_ID)

---

### 2. Implement Real Solana Transactions in Processor

**Current Code (Simulated):**
```rust
// services/processor/src/worker_pool.rs:270
let signature = format!("SIM_{}", Uuid::new_v4());
```

**Need to Replace With:**

```rust
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    transaction::Transaction,
};
use anchor_lang::InstructionData;

async fn submit_batch_to_solana(
    &self,
    bets: Vec<Bet>,
) -> Result<(String, Vec<(Uuid, bool, u64)>)> {
    let client = self.solana_client.get_healthy_client().await
        .ok_or_else(|| anyhow::anyhow!("No healthy RPC clients"))?;

    // Build instructions for each bet
    let mut instructions = Vec::new();
    let mut results = Vec::new();
    
    for bet in &bets {
        // Derive user vault PDA
        let (user_vault, _) = Pubkey::find_program_address(
            &[b"user_vault", bet.user_wallet.as_bytes()],
            &self.vault_program_id,
        );
        
        // Derive casino vault PDA
        let (casino_vault, _) = Pubkey::find_program_address(
            &[b"casino_vault"],
            &self.vault_program_id,
        );
        
        // Determine bet result
        let won = simulate_coinflip();
        let payout = if won { bet.stake_amount * 2 } else { 0 };
        results.push((bet.bet_id, won, payout));
        
        // Build spend_from_allowance instruction
        let ix_data = vault::instruction::SpendFromAllowance {
            amount: bet.stake_amount,
            bet_id: bet.bet_id.to_string(),
        };
        
        let spend_ix = Instruction {
            program_id: self.vault_program_id,
            accounts: vec![
                AccountMeta::new(user_vault, false),
                AccountMeta::new(casino_vault, false),
                AccountMeta::new_readonly(self.processor_keypair.pubkey(), true),
            ],
            data: ix_data.data(),
        };
        instructions.push(spend_ix);
        
        // If won, add payout instruction
        if won {
            let payout_ix = vault::instruction::Payout {
                amount: payout,
                bet_id: bet.bet_id.to_string(),
            };
            
            let payout_ix = Instruction {
                program_id: self.vault_program_id,
                accounts: vec![
                    AccountMeta::new(casino_vault, false),
                    AccountMeta::new(user_vault, false),
                    AccountMeta::new_readonly(self.processor_keypair.pubkey(), true),
                ],
                data: payout_ix.data(),
            };
            instructions.push(payout_ix);
        }
    }
    
    // Build transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&self.processor_keypair.pubkey()),
        &[&self.processor_keypair],
        recent_blockhash,
    );
    
    // Send and confirm
    let signature = client.send_and_confirm_transaction(&transaction)?;
    
    tracing::info!("Solana transaction confirmed: {}", signature);
    
    Ok((signature.to_string(), results))
}
```

---

### 3. Frontend Wallet Integration

**Current:** Mock data  
**Need:** Real Solana calls

**Install SDK:**
```bash
cd apps/frontend
pnpm add @coral-xyz/anchor @solana/web3.js
```

**Create Vault SDK:**
```typescript
// apps/frontend/src/lib/vault-sdk.ts
import { Program, AnchorProvider } from '@coral-xyz/anchor';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import type { Vault } from '../idl/vault';
import idl from '../idl/vault.json';

export class VaultSDK {
  constructor(
    private program: Program<Vault>,
    private connection: Connection
  ) {}

  // Derive user vault PDA
  async getUserVaultAddress(userPubkey: PublicKey): Promise<PublicKey> {
    const [vaultPda] = await PublicKey.findProgramAddress(
      [Buffer.from('user_vault'), userPubkey.toBuffer()],
      this.program.programId
    );
    return vaultPda;
  }

  // Check if vault exists
  async vaultExists(userPubkey: PublicKey): Promise<boolean> {
    const vaultAddress = await this.getUserVaultAddress(userPubkey);
    const account = await this.connection.getAccountInfo(vaultAddress);
    return account !== null;
  }

  // Initialize vault
  async initializeVault(userPubkey: PublicKey) {
    const vaultAddress = await this.getUserVaultAddress(userPubkey);
    
    return await this.program.methods
      .initializeVault()
      .accounts({
        userVault: vaultAddress,
        user: userPubkey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  // Deposit SOL
  async depositSol(userPubkey: PublicKey, amount: number) {
    const vaultAddress = await this.getUserVaultAddress(userPubkey);
    
    return await this.program.methods
      .depositSol(new BN(amount))
      .accounts({
        userVault: vaultAddress,
        user: userPubkey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  // Approve allowance
  async approveAllowance(
    userPubkey: PublicKey,
    amount: number,
    durationSeconds: number,
    tokenMint: PublicKey
  ) {
    const vaultAddress = await this.getUserVaultAddress(userPubkey);
    
    return await this.program.methods
      .approveAllowance(
        new BN(amount),
        new BN(durationSeconds),
        tokenMint
      )
      .accounts({
        userVault: vaultAddress,
        user: userPubkey,
      })
      .rpc();
  }

  // Get vault balance
  async getBalance(userPubkey: PublicKey) {
    const vaultAddress = await this.getUserVaultAddress(userPubkey);
    const account = await this.program.account.userVault.fetch(vaultAddress);
    return {
      sol: account.solBalance.toNumber(),
      usdc: account.splBalance.toNumber(),
    };
  }

  // Get allowance
  async getAllowance(userPubkey: PublicKey) {
    const vaultAddress = await this.getUserVaultAddress(userPubkey);
    const account = await this.program.account.userVault.fetch(vaultAddress);
    
    if (!account.allowanceActive) {
      return null;
    }
    
    return {
      amount: account.allowanceAmount.toNumber(),
      remaining: account.allowanceRemaining.toNumber(),
      expiresAt: new Date(account.allowanceExpiresAt.toNumber() * 1000),
    };
  }
}
```

**Update VaultDashboard.tsx:**
```typescript
'use client';

import { useEffect, useState } from 'react';
import { useWallets } from '@privy-io/react-auth';
import { Connection, PublicKey } from '@solana/web3.js';
import { VaultSDK } from '@/lib/vault-sdk';

export function VaultDashboard() {
  const { wallets } = useWallets();
  const [sdk, setSdk] = useState<VaultSDK | null>(null);
  const [balance, setBalance] = useState({ sol: 0, usdc: 0 });
  const [allowance, setAllowance] = useState(null);

  useEffect(() => {
    if (wallets[0]) {
      // Initialize SDK
      const connection = new Connection(process.env.NEXT_PUBLIC_SOLANA_RPC_URL);
      // ... setup program and SDK
    }
  }, [wallets]);

  const handleDeposit = async () => {
    // Call sdk.depositSol()
  };

  const handleApprove = async () => {
    // Call sdk.approveAllowance()
  };

  // ... rest of component
}
```

---

### 4. Update Backend to Validate Allowances

**Add Solana Validation:**
```rust
// services/backend/src/handlers/bets.rs

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

async fn create_bet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateBetRequest>,
) -> Result<Json<CreateBetResponse>, AppError> {
    // Get user wallet from header
    let user_wallet = headers
        .get("X-User-Wallet")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::BadRequest("Missing X-User-Wallet header".into()))?;

    // Verify vault exists and has active allowance
    let vault_program_id = Pubkey::from_str(&state.config.solana.vault_program_id)?;
    let user_pubkey = Pubkey::from_str(user_wallet)?;
    
    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"user_vault", user_pubkey.to_bytes()],
        &vault_program_id,
    );
    
    // Fetch vault account
    let client = RpcClient::new(&state.config.solana.rpc_url);
    let vault_account = client.get_account(&vault_pda)
        .map_err(|_| AppError::BadRequest("Vault not found".into()))?;
    
    // Deserialize and check allowance
    // let vault: UserVault = try_from_slice_unchecked(&vault_account.data)?;
    // if !vault.allowance_active {
    //     return Err(AppError::BadRequest("No active allowance".into()));
    // }
    // if vault.allowance_remaining < request.stake_amount {
    //     return Err(AppError::BadRequest("Insufficient allowance".into()));
    // }

    // Continue with bet creation...
}
```

---

## 🔧 Step-by-Step Implementation Plan

### Phase 1: Deploy Program (30 min)
1. Install Anchor CLI
2. Build vault program
3. Deploy to devnet
4. Update all program IDs in config files
5. Fund processor keypair: `solana airdrop 2 <PROCESSOR_PUBKEY>`

### Phase 2: Processor Integration (2-3 hours)
1. Remove simulation code
2. Implement real transaction building
3. Add proper error handling
4. Test with 1-2 bets first
5. Add transaction retry logic

### Phase 3: Frontend Integration (3-4 hours)
1. Generate TypeScript IDL: `anchor build && anchor idl parse`
2. Copy IDL to frontend: `apps/frontend/src/idl/vault.json`
3. Implement VaultSDK class
4. Update VaultDashboard with real queries
5. Add transaction signing UI
6. Test vault initialization
7. Test deposit flow
8. Test allowance approval
9. Test bet placement

### Phase 4: Backend Validation (1-2 hours)
1. Add Solana RPC client to backend
2. Implement allowance checking
3. Add vault existence validation
4. Update error messages

### Phase 5: Testing (2-3 hours)
1. Test full flow: deposit → approve → bet → process → payout
2. Test edge cases: expired allowance, insufficient balance
3. Test concurrent bets
4. Monitor transaction success rates

---

## 💰 Costs to Consider

### Devnet (Free)
- Unlimited SOL via faucet
- Perfect for testing
- No real money

### Mainnet (Real Costs)
- **Program Deployment:** ~2-5 SOL one-time
- **Rent per vault:** ~0.002 SOL per user vault
- **Transaction fees:** ~0.000005 SOL per transaction
- **Processor needs:** ~1-2 SOL for transaction fees

**Recommendation:** Stay on devnet until fully tested

---

## 🔐 Security Checklist Before Mainnet

- [ ] External security audit of Anchor program
- [ ] Fuzz testing of program instructions
- [ ] Rate limiting on allowance approvals
- [ ] Maximum bet size enforcement
- [ ] Emergency pause mechanism tested
- [ ] Processor keypair in secure HSM/KMS
- [ ] Transaction monitoring and alerts
- [ ] Backup RPC endpoints configured

---

## 📊 Expected Performance After Integration

| Metric | Devnet | Mainnet |
|--------|--------|---------|
| Transaction confirmation | 10-30s | 5-15s |
| Batch size | 10-20 bets | 5-10 bets (compute limit) |
| Cost per bet | Free | ~$0.00001 |
| Failed tx rate | 5-10% | 1-3% |

---

## 🚨 Current Limitations

### Without Blockchain:
- ❌ No actual fund custody
- ❌ Can't verify user balances
- ❌ Can't enforce allowances
- ❌ No on-chain transparency
- ❌ Users could dispute results

### With Blockchain:
- ✅ Real fund custody in user PDAs
- ✅ Allowances enforced on-chain
- ✅ All bets verifiable on Solana
- ✅ Automatic payouts
- ✅ Users maintain self-custody

---

## 📝 Quick Start Commands

```bash
# 1. Deploy program
cd programs/vault
anchor build
anchor deploy --provider.cluster devnet

# 2. Fund processor
solana airdrop 2 $(solana-keygen pubkey test-keypair.json)

# 3. Update configs with program ID
anchor keys list
# Copy program ID to all .env files

# 4. Test with frontend
cd ../../apps/frontend
pnpm install
pnpm dev

# 5. Test full flow
# - Connect wallet
# - Initialize vault
# - Deposit 1 SOL
# - Approve 0.5 SOL allowance
# - Place bet
# - Check processor logs for real TX
```

---

## 🎯 Success Criteria

You'll know it's working when:
1. ✅ Frontend shows real vault balance from blockchain
2. ✅ Allowance approval creates on-chain transaction
3. ✅ Placing bet doesn't require wallet signature
4. ✅ Processor logs show real Solana TX IDs (not SIM_)
5. ✅ Can view transactions on Solana Explorer
6. ✅ Payouts appear in vault automatically
7. ✅ Withdrawals transfer SOL back to wallet

---

## 📚 Additional Resources

- **Anchor Docs:** https://www.anchor-lang.com/
- **Solana Cookbook:** https://solanacookbook.com/
- **Program Examples:** https://github.com/coral-xyz/anchor/tree/master/tests
- **Solana Explorer:** https://explorer.solana.com/?cluster=devnet
