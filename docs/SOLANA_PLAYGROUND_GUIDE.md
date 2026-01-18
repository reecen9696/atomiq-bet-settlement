# Solana Playground Deployment Guide

## Why Playground?
- No local build issues (builds in cloud)
- No disk space needed
- No Anchor version conflicts
- Deploys in ~5 minutes

## Step-by-Step Instructions

### 1. Open Solana Playground
Go to: **https://beta.solpg.io**

### 2. Create New Project
- Click "+ Create a new project"
- Select "Anchor" framework
- Name it "vault" or anything you like

### 3. Copy Your Program Code

The playground will create default files. Replace them with yours:

#### File: `lib.rs`
Copy contents from: `/Users/reece/code/projects/atomik-wallet/programs/vault/src/lib.rs`

#### File: `state.rs` (create new file)
Copy contents from: `/Users/reece/code/projects/atomik-wallet/programs/vault/src/state.rs`

#### File: `errors.rs` (create new file)
Copy contents from: `/Users/reece/code/projects/atomik-wallet/programs/vault/src/errors.rs`

#### File: `Cargo.toml`
Replace with:
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
- Click the "Build" button (hammer icon)
- Wait for build to complete (~2-3 minutes)
- You should see "Build successful"

### 5. Deploy to Devnet
- Click "Deploy" button
- Select "Devnet" if asked
- Click "Deploy"
- **IMPORTANT**: Copy the Program ID from the output

### 6. Update Your Local Config

Run these commands with YOUR program ID:

```bash
# Replace <YOUR_PROGRAM_ID> with the ID from step 5
cd /Users/reece/code/projects/atomik-wallet

# Update configs
sed -i '' 's/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=<YOUR_PROGRAM_ID>/' services/backend/.env
sed -i '' 's/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=<YOUR_PROGRAM_ID>/' services/processor/.env

# Enable real Solana
sed -i '' 's/USE_REAL_SOLANA=false/USE_REAL_SOLANA=true/' services/processor/.env

# Fund processor wallet
solana airdrop 2 $(solana-keygen pubkey test-keypair.json) --url devnet
```

### 7. Test Real Transactions

Start your services:

```bash
# Terminal 1: Backend
cd services/backend
cargo run

# Terminal 2: Processor
cd services/processor
cargo run

# Terminal 3: Test
./test-system.sh
```

## Current Configuration

Your local files are already at Anchor 0.30.1:
- `programs/vault/Cargo.toml` - anchor-lang = "0.30.1"
- `programs/vault/Cargo.toml` - anchor-spl = "0.30.1"

So you can copy these files directly to Playground without changes.

## Troubleshooting

**Q: Build fails in Playground**
- Check that all three files (lib.rs, state.rs, errors.rs) are present
- Verify Cargo.toml has correct dependencies
- Try refreshing the page and rebuilding

**Q: Deploy fails**
- Make sure you selected "Devnet"
- Check that you have SOL in your Playground wallet (it auto-airdrops)
- Try again - devnet can be slow sometimes

**Q: Can't copy program ID**
- Look for output like: `Program Id: 8Xy9...abc123`
- Copy everything after "Program Id: "

## Alternative: Local Build (if you want to try later)

If you want to build locally later, you'll need to:
1. Free up more disk space (need 15-20GB free)
2. Fix Anchor CLI installation issues
3. Or use Docker to isolate the build

But Playground is much faster right now!
