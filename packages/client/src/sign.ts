import { TransactionSkeletonType } from "@ckb-lumos/helpers";
import ECPair from '@nervosnetwork/ckb-sdk-utils/lib/ecpair'

async function signWithPriv(skeleton: TransactionSkeletonType): Promise<Array<string>> {
  const sk = process.env.SK;
  if (!sk) throw new Error("No private key provided in environment!");

  const keyPair = new ECPair(sk);

  const signingEntries = skeleton.get("signingEntries").toArray();
  return signingEntries.map(entry => keyPair.signRecoverable(entry.message));
}

export { signWithPriv };
