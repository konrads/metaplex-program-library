/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import { MetadataArgs, metadataArgsBeet } from '../types/MetadataArgs'

/**
 * @category Instructions
 * @category MintV1
 * @category generated
 */
export type MintV1InstructionArgs = {
  message: MetadataArgs
}
/**
 * @category Instructions
 * @category MintV1
 * @category generated
 */
export const mintV1Struct = new beet.FixableBeetArgsStruct<
  MintV1InstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['message', metadataArgsBeet],
  ],
  'MintV1InstructionArgs'
)
/**
 * Accounts required by the _mintV1_ instruction
 *
 * @property [_writable_] treeAuthority
 * @property [] leafOwner
 * @property [] leafDelegate
 * @property [_writable_] merkleTree
 * @property [**signer**] payer
 * @property [**signer**] treeDelegate
 * @property [] logWrapper
 * @property [] compressionProgram
 * @category Instructions
 * @category MintV1
 * @category generated
 */
export type MintV1InstructionAccounts = {
  treeAuthority: web3.PublicKey
  leafOwner: web3.PublicKey
  leafDelegate: web3.PublicKey
  merkleTree: web3.PublicKey
  payer: web3.PublicKey
  treeDelegate: web3.PublicKey
  logWrapper: web3.PublicKey
  compressionProgram: web3.PublicKey
}

export const mintV1InstructionDiscriminator = [
  145, 98, 192, 118, 184, 147, 118, 104,
]

/**
 * Creates a _MintV1_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category MintV1
 * @category generated
 */
export function createMintV1Instruction(
  accounts: MintV1InstructionAccounts,
  args: MintV1InstructionArgs,
  programId = new web3.PublicKey('BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY')
) {
  const [data] = mintV1Struct.serialize({
    instructionDiscriminator: mintV1InstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.treeAuthority,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.leafOwner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.leafDelegate,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.merkleTree,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.payer,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.treeDelegate,
      isWritable: false,
      isSigner: true,
    },
    {
      pubkey: accounts.logWrapper,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.compressionProgram,
      isWritable: false,
      isSigner: false,
    },
  ]

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}
