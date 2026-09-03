use crate::error::CoreError;

/// A single command in the QCY `0xFF`-framed binary protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub opcode: u8,
    pub parameters: Vec<u8>,
}

impl Command {
    pub fn new(opcode: u8, parameters: Vec<u8>) -> Self {
        Self { opcode, parameters }
    }

    /// Packs the command into the QCY wire frame:
    /// `[0xFF, BodyLength, Opcode, ParamLength, Params...]`.
    pub fn pack(&self) -> Vec<u8> {
        let mut body = vec![self.opcode, self.parameters.len() as u8];
        body.extend(&self.parameters);

        let mut packet = vec![0xFF, body.len() as u8];
        packet.extend(body);
        packet
    }

    /// Parses a raw byte slice into one or more [`Command`]s.
    pub fn parse(packet: &[u8]) -> Result<Vec<Command>, CoreError> {
        if packet.len() < 4 || packet[0] != 0xFF {
            return Err(CoreError::InvalidPacket("Too short or missing SOF".into()));
        }

        let body_len = packet[1] as usize;
        if body_len + 2 != packet.len() {
            return Err(CoreError::InvalidPacket("Body length mismatch".into()));
        }

        let mut commands = Vec::new();
        let mut offset = 2;

        while offset < packet.len() {
            if offset + 2 > packet.len() {
                return Err(CoreError::InvalidPacket("Truncated command block".into()));
            }
            let opcode = packet[offset];
            let param_len = packet[offset + 1] as usize;
            offset += 2;

            if offset + param_len > packet.len() {
                return Err(CoreError::InvalidPacket("Truncated parameters".into()));
            }

            let parameters = packet[offset..offset + param_len].to_vec();
            commands.push(Command::new(opcode, parameters));
            offset += param_len;
        }

        if commands.is_empty() {
            return Err(CoreError::InvalidPacket("No commands found".into()));
        }

        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_frame_correctly() {
        let cmd = Command::new(0x17, vec![0x02, 0x00, 0x00]);
        assert_eq!(cmd.pack(), vec![0xFF, 0x05, 0x17, 0x03, 0x02, 0x00, 0x00]);
    }

    #[test]
    fn parses_single_command() {
        let packet = [0xFF, 0x05, 0x17, 0x03, 0x02, 0x00, 0x00];
        let commands = Command::parse(&packet).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].opcode, 0x17);
        assert_eq!(commands[0].parameters, vec![0x02, 0x00, 0x00]);
    }

    #[test]
    fn rejects_missing_sof() {
        assert!(Command::parse(&[0x00, 0x05, 0x17, 0x03, 0x02, 0x00, 0x00]).is_err());
    }

    #[test]
    fn rejects_body_length_mismatch() {
        assert!(Command::parse(&[0xFF, 0x09, 0x17, 0x03, 0x02, 0x00, 0x00]).is_err());
    }
}
