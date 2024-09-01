import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import {
    PublicKey,
    Connection,
    TransactionInstruction,
    VersionedTransaction,
    TransactionMessage,
} from "@solana/web3.js";
import { BN } from "bn.js";
import { TokenEscrow } from "../types/token_escrow";
import * as IDL from "../types/token_escrow.json";

export const idl: TokenEscrow = IDL as TokenEscrow;

const ESCROW_SEED: string = "escrow";

export function getEscrowAddress(authority: PublicKey, tokenA: PublicKey, tokenB: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
        [
            Buffer.from(ESCROW_SEED),
            authority.toBuffer(),
            tokenA.toBuffer(),
            tokenB.toBuffer(),
        ],
        new PublicKey(idl.address),
    );
}

export type InitFields = {
    to: PublicKey,
    tokenA: PublicKey,
    tokenB: PublicKey,
    amountA: number,
    amountB: number,
    expiry: number,
}

export class TokenEscrowProgram {
    program: Program<TokenEscrow>;

    constructor(connection: Connection, wallet: Wallet) {
        const provider = new AnchorProvider(
            connection,
            wallet,
            {
                preflightCommitment: "confirmed",
                commitment: "confirmed",
            }
        );
        this.program = new Program(idl, provider);
    }

    async sendTransaction(instructions: TransactionInstruction[]): Promise<string> {
        const transaction = await this.prepareTransaction(instructions);
        const txid = await this.program.provider.sendAndConfirm(transaction);

        return txid;
    }

    async prepareTransaction(instructions: TransactionInstruction[]): Promise<VersionedTransaction> {
        const maxRetries = 3;
        const delay = 1000;
        let retries = 0;

        while (retries > maxRetries) {
            try {
                const recentBlockhash = await this.program
                    .provider
                    .connection
                    .getLatestBlockhash()
                    .then(r => r.blockhash);

                const messageV0 = new TransactionMessage({
                    payerKey: this.program.provider.publicKey,
                    recentBlockhash,
                    instructions,
                }).compileToV0Message();

                return new VersionedTransaction(messageV0);
            } catch (error) {
                console.error("Could not get blockhash:", error)
                retries++;
                if (retries < maxRetries) {
                    await new Promise(resolve => setTimeout(resolve, delay));
                } else {
                    throw error;
                }
            }
        }
    }

    getInitIx(fields: InitFields): Promise<TransactionInstruction> {
        const from = this.program.provider.publicKey;

        return this.program.methods.init({
            to: fields.to,
            tokenB: fields.tokenB,
            amountA: new BN(fields.amountA),
            amountB: new BN(fields.amountB),
            expiry: new BN(fields.expiry),
        }).accounts({
            from,
            tokenA: fields.tokenA,
        }).instruction();
    }

    getTransferIx(tokenA: PublicKey, tokenB: PublicKey, recipient: PublicKey): Promise<TransactionInstruction> {
        const payer = this.program.provider.publicKey;
        return this.program.methods.transfer().accounts({
            tokenA,
            tokenB,
            payer,
            recipient,
        }).instruction();
    }

    getCancelIx(tokenA: PublicKey, tokenB: PublicKey): Promise<TransactionInstruction> {
        const authority = this.program.provider.publicKey;
        return this.program.methods.cancel(tokenB).accounts({
            authority,
            tokenA,
        }).instruction();
    }
}
