// Data type structures needed

export interface LockReceipt {
  isLock: boolean,
  user: string,
  amount: string,
  txHash: string
}

export interface UnlockReceipt {
  user: string,
  amount: string,
  txHash: string,
  sigs: string,
}