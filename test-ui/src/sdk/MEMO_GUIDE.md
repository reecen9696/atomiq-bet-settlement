# 🎯 Transaction Descriptions with Memo Instructions

The Atomik SDK includes **descriptive memo instructions** in all transactions that appear in wallet popups like Phantom. This provides clear user communication about what each transaction does.

## How It Works

When you call SDK functions like `vault.deposit(0.5)`, the SDK automatically:

1. Creates a memo instruction with a human-readable message
2. Adds it as the **first instruction** in the transaction
3. Adds your actual program instruction(s) after it
4. Users see the description in their wallet approval popup

## Wallet Message Examples

| Action                           | Message in Phantom Wallet                                              |
| -------------------------------- | ---------------------------------------------------------------------- |
| `vault.initialize()`             | "Initialize your Atomik vault for secure betting"                      |
| `vault.deposit(0.5)`             | "Deposit 0.5 SOL to your Atomik vault"                                 |
| `vault.withdraw(1.0)`            | "Withdraw 1.0 SOL from your Atomik vault"                              |
| `allowance.approve(2.0)`         | "Approve session key to spend up to 2.0 SOL for bets until 2026-02-05" |
| `allowance.revoke()`             | "Revoke session key spending permission"                               |
| `betting.placeBet('heads', 0.1)` | "Bet 0.1 SOL on heads - Coinflip Game"                                 |
| `betting.settleGame('abc123')`   | "Settle game abc123 and claim winnings"                                |

## Technical Implementation

### Memo Program ID

```typescript
export const MEMO_PROGRAM_ID = new PublicKey(
  "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
);
```

### Creating Memo Instructions

```typescript
import { createMemoInstruction, MemoMessages } from "./sdk";

// Use predefined messages
const memoInstruction = createMemoInstruction(MemoMessages.depositSol(1.5));

// Or create custom messages
const customMemo = createMemoInstruction("Custom transaction description");
```

### Building Transactions with Memos

**Important**: Always add memo instruction FIRST so it's prominent in wallet UI.

```typescript
import { Transaction, SystemProgram } from "@solana/web3.js";
import { createMemoInstruction, MemoMessages } from "./sdk/utils/memo";

function buildDepositTransaction(userPubkey, vaultPda, amount) {
  const transaction = new Transaction();

  // 1. Add memo instruction FIRST
  const memoInstruction = createMemoInstruction(
    MemoMessages.depositSol(amount),
  );
  transaction.add(memoInstruction);

  // 2. Add actual program instruction(s) after memo
  const transferInstruction = SystemProgram.transfer({
    fromPubkey: userPubkey,
    toPubkey: vaultPda,
    lamports: amount * 1e9,
  });
  transaction.add(transferInstruction);

  return transaction;
}
```

### Complete Implementation Examples

The SDK includes complete implementation examples in:

- `src/sdk/examples/memo-transactions.ts` - Full transaction building examples
- `src/sdk/utils/memo.ts` - Memo utility functions and predefined messages
- `src/components/VaultManagerSDK.tsx` - React component usage

## User Experience Benefits

### Before (without memos):

- Users see generic transaction popup: "Sign Transaction"
- No context about what the transaction does
- Users may hesitate or cancel due to uncertainty

### After (with memos):

- Users see clear description: "Deposit 0.5 SOL to your Atomik vault"
- Immediate understanding of transaction purpose
- Increased confidence and approval rates

## Best Practices

### 1. Keep Messages Concise

```typescript
// ✅ Good - clear and concise
"Deposit 0.5 SOL to your vault";

// ❌ Too verbose - wallets may truncate
"This transaction will deposit 0.5 SOL from your wallet to your Atomik casino vault account for future betting operations";
```

### 2. Include Key Information

```typescript
// ✅ Include amounts, choices, dates
MemoMessages.placeBet("heads", 0.1); // "Bet 0.1 SOL on heads - Coinflip Game"
MemoMessages.approveAllowance(1.0, "2026-02-05"); // "Approve session key to spend up to 1.0 SOL for bets until 2026-02-05"

// ❌ Too generic
("Place a bet");
```

### 3. Order Instructions Correctly

```typescript
// ✅ Memo first, program instructions after
transaction.add(memoInstruction);
transaction.add(programInstruction);

// ❌ Memo buried in middle
transaction.add(programInstruction);
transaction.add(memoInstruction);
transaction.add(anotherInstruction);
```

## Security Notes

- Memo text is **public on-chain** - never include secrets
- Messages are **immutable** once on-chain
- Keep messages **user-friendly** but **informative**
- Test memo display in different wallets (Phantom, Solflare, etc.)

## Integration Checklist

- [ ] Import memo utilities: `import { createMemoInstruction, MemoMessages } from './sdk'`
- [ ] Add memo instruction as **first instruction** in all user transactions
- [ ] Use predefined `MemoMessages` for consistency
- [ ] Test memo display in wallet popups
- [ ] Verify memo text is helpful and not too long
- [ ] Include relevant transaction details (amounts, choices, dates)

## Future Enhancements

Consider adding:

- [ ] Multi-language memo support
- [ ] Dynamic memo generation based on user preferences
- [ ] Memo templates for custom game types
- [ ] Integration with wallet-specific memo formatting
