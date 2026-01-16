# Quick Deploy Instructions - Solana Playground

## Why This Approach?
- **ALL** local Anchor CLI versions (0.27-0.32) have yanked `cargo_toml` dependency - cannot install
- `cargo build-sbf` requires edition2024 support (Cargo 1.90.0+) but Solana's BPF toolchain uses Cargo 1.84.0
- Solana Playground builds in cloud → **works immediately**

## 5-Minute Deployment Steps

### 1. Open https://beta.solpg.io in your browser

### 2. Create New Project
- Click "+ Create a new project"
- Select "Anchor" framework  
- Name: "vault"

### 3. Replace Files

The playground creates default files. You need to replace/create these:

#### **lib.rs** (main program file)
```bash
# On your Mac, copy this file:
cat /Users/reece/code/projects/atomik-wallet/programs/vault/src/lib.rs
```
Copy the output and paste into Playground's `lib.rs`

#### **state.rs** (click "+ New File" in Playground)
```bash
# Copy this file:
cat /Users/reece/code/projects/atomik-wallet/programs/vault/src/state.rs
```
Create new file `state.rs` and paste

#### **errors.rs** (click "+ New File" in Playground)
```bash
# Copy this file:
cat /Users/reece/code/projects/atomik-wallet/programs/vault/src/errors.rs
```
Create new file `errors.rs` and paste

#### **Cargo.toml** (update existing)
```toml
[package]
name = "vault"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "vault"

[dependencies]
anchor-lang = "0.30.1"
anchor-spl = "0.30.1"

[profile.release]
overflow-checks = true
```

### 4. Build in Playground
- Click "Build" button (hammer icon on left sidebar)
- Wait 2-3 minutes
- Should see "✓ Build successful"

### 5. Deploy to Devnet  
- Click "Deploy" button
- Select "Devnet" network
- Click "Deploy"
- **COPY THE PROGRAM ID** from output (looks like: `8Xy9abc...123`)

### 6. Update Your Local Config

Run these commands on your Mac:

```bash
cd /Users/reece/code/projects/atomik-wallet

# Replace <YOUR_PROGRAM_ID> with the ID from step 5
export NEW_PROGRAM_ID="<YOUR_PROGRAM_ID>"

# Update backend config
sed -i '' "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" services/backend/.env

# Update processor config  
sed -i '' "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" services/processor/.env

# Enable real Solana transactions
sed -i '' 's/USE_REAL_SOLANA=false/USE_REAL_SOLANA=true/' services/processor/.env

# Fund the processor wallet
solana airdrop 2 $(solana-keygen pubkey test-keypair.json) --url devnet

# Verify configs updated
echo "Backend program ID:"
grep VAULT_PROGRAM_ID services/backend/.env
echo "Processor program ID:"
grep VAULT_PROGRAM_ID services/processor/.env
echo "Processor using real Solana:"
grep USE_REAL_SOLANA services/processor/.env
```

### 7. Test Real Blockchain Transactions

```bash
# Terminal 1: Start backend
cd /Users/reece/code/projects/atomik-wallet/services/backend
cargo run

# Terminal 2: Start processor  
cd /Users/reece/code/projects/atomik-wallet/services/processor
cargo run

# Terminal 3: Run test
cd /Users/reece/code/projects/atomik-wallet
./test-system.sh
```

## For Future Updates

When you need to deploy changes:

1. **Make code changes locally** in `programs/vault/src/`
2. **Copy updated files to Playground**
3. **Click "Build"** in Playground
4. **Click "Deploy"** (can deploy to same program ID)
5. **No local config changes needed** (program ID stays same)

This gives you rapid iteration with CLI-like workflow!

## Alternative: Pre-built Binary (IF Someone Provides)

If someone gives you a pre-built `vault.so` file:

```bash
# Deploy directly
solana program deploy vault.so --url devnet --program-id <YOUR_PROGRAM_ID>

# Update configs as shown in step 6
```

## Troubleshooting

**Q: Build fails in Playground**
- Make sure all 3 source files are present (lib.rs, state.rs, errors.rs)
- Check Cargo.toml has correct dependencies
- Try refreshing page and rebuilding

**Q: Deploy fails**  
- Playground auto-airdrops SOL to your wallet
- If it fails, wait 30 seconds and try again (devnet can be slow)

**Q: Program ID different each time I deploy**
- That's normal for first deploy
- Subsequent deploys can use same ID with `--program-id` flag
- Or just update configs each time (only takes 10 seconds)

## Why Local Build Failed

The issue blocking ALL local builds:

1. Anchor 0.27, 0.28 require `cargo_toml ^0.13.0` → **ALL yanked** from crates.io
2. Anchor 0.29-0.30 have Solana SDK version conflicts  
3. Anchor 0.30.1+ pull `constant_time_eq v0.4.2` → requires edition2024 (Rust 1.90.0)
4. Solana's `cargo build-sbf` uses pinned Cargo 1.84.0 → **no edition2024 support**

Result: **Impossible to build locally** without Docker or VM with specific versions.

Solana Playground avoids ALL these issues → **always works**!
