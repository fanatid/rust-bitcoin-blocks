use bytes::Buf;

use super::fixed_hash::H256;

trait BufExtra: Buf {
    fn get_varuint(&mut self) -> u64 {
        let first = self.get_u8();
        match first {
            0xfd => self.get_u16_le() as u64,
            0xfe => self.get_u32_le() as u64,
            0xff => self.get_u64_le(),
            _ => first as u64,
        }
    }

    fn get_varuint_slice(&mut self) -> Vec<u8> {
        let size = self.get_varuint() as usize;
        let mut slice = vec![0; size];
        self.copy_to_slice(slice.as_mut_slice());
        slice
    }

    fn get_varuint_slice_vec(&mut self) -> Vec<Vec<u8>> {
        let count = self.get_varuint() as usize;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(self.get_varuint_slice());
        }
        vec
    }
}

impl BufExtra for &[u8] {}

pub trait StructFromBytes {
    type Output;

    fn from_bytes(bytes: &mut &[u8]) -> Self::Output;

    fn from_bytes_varuint_vec(bytes: &mut &[u8]) -> Vec<Self::Output> {
        let count = bytes.get_varuint() as usize;
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(Self::from_bytes(bytes));
        }
        vec
    }
}

#[derive(Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl StructFromBytes for Block {
    type Output = Block;

    fn from_bytes(bytes: &mut &[u8]) -> Self::Output {
        Block {
            header: BlockHeader::from_bytes(bytes),
            transactions: Transaction::from_bytes_varuint_vec(bytes),
        }
    }
}

#[derive(Debug)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_hash: H256,
    pub merkle_root: H256,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
}

impl StructFromBytes for BlockHeader {
    type Output = BlockHeader;

    fn from_bytes(bytes: &mut &[u8]) -> Self::Output {
        BlockHeader {
            version: bytes.get_u32_le(),
            prev_hash: H256::from_bytes(bytes),
            merkle_root: H256::from_bytes(bytes),
            timestamp: bytes.get_u32_le(),
            bits: bytes.get_u32_le(),
            nonce: bytes.get_u32_le(),
        }
    }
}

#[derive(Debug)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub locktime: u32,
}

impl StructFromBytes for Transaction {
    type Output = Transaction;

    fn from_bytes(bytes: &mut &[u8]) -> Self::Output {
        let version = bytes.get_u32_le();

        let mut flags = 0;
        let mut inputs = TransactionInput::from_bytes_varuint_vec(bytes);
        let outputs = if inputs.is_empty() {
            flags = bytes.get_u8();
            inputs = TransactionInput::from_bytes_varuint_vec(bytes);
            TransactionOutput::from_bytes_varuint_vec(bytes)
        } else {
            TransactionOutput::from_bytes_varuint_vec(bytes)
        };

        if (flags & 1) == 1 {
            for input in inputs.iter_mut() {
                input.witness = Some(bytes.get_varuint_slice_vec());
            }
        }

        Transaction {
            version,
            inputs,
            outputs,
            locktime: bytes.get_u32_le(),
        }
    }
}

#[derive(Debug)]
pub struct TransactionInput {
    pub hash: H256,
    pub index: u32,
    pub script: Vec<u8>,
    pub sequence: u32,
    pub witness: Option<Vec<Vec<u8>>>,
}

impl StructFromBytes for TransactionInput {
    type Output = TransactionInput;

    fn from_bytes(bytes: &mut &[u8]) -> Self::Output {
        TransactionInput {
            hash: H256::from_bytes(bytes),
            index: bytes.get_u32_le(),
            script: bytes.get_varuint_slice(),
            sequence: bytes.get_u32_le(),
            witness: None,
        }
    }
}

#[derive(Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub script: Vec<u8>,
}

impl StructFromBytes for TransactionOutput {
    type Output = TransactionOutput;

    fn from_bytes(bytes: &mut &[u8]) -> Self::Output {
        TransactionOutput {
            value: bytes.get_u64_le(),
            script: bytes.get_varuint_slice(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tx() {
        let txs = [
            "020000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff4d0360820904ab2d7e5e322f537069646572506f6f6cfabe6d6d664b0f8173a95a4ac375a977f76bcfe417f01b90097c8cd6245774e00e2f0af9020000008df9e4830ec0434099eb0e0000000000ffffffff0396f8fb4c0000000017a9144f0dd555d6f74016b8c57b265b17615dc092fa76870000000000000000266a24aa21a9ed1a90d574bff31c8aaf87630560799d1d949f5d9c7a87722a831dcf9ebb24f3ed0000000000000000266a24b9e11b6d93c98d94ff4693be57b5d99f1f4febb5dd1e867c4ed081c70059fd97f3c5e8df0120000000000000000000000000000000000000000000000000000000000000000000000000",
            "02000000014d8004262a6686f9daa8935f68ee6bfc9363b4e162b8696e5e891af9f77a2947000000006b483045022100d957a36572fcf69714ca4f5ef014959a9945b59c416e30ba1904e9e18d9db848022008d7aab903ade3b64036d3e2513f5708b2589762c822f68e8f54c3398a91c3d7012103a0c53fcc4704ba78331a896c3bd684328b44890b25f91cfb853ab0bb301c7875ffffffff0490350f00000000001976a914b0f90d990d7c41d95128d6f2f74384e5514b05e788ac51281d00000000001976a9142ab4e140a02e677a388e5f54798445398d7ebf6188ac3df445160000000017a914015bd7fecbd9d1e2ae5eb595c4de144a5679f4c1875f671509000000001976a91443849383122ebb8a28268a89700c9f723663b5b888ac00000000",
        ];

        for tx_data in txs.iter() {
            let bytes = hex::decode(tx_data).unwrap();
            let slice = &mut &bytes[..];
            let _tx = Transaction::from_bytes(slice);
            assert_eq!(slice.remaining(), 0);
        }
    }
}
