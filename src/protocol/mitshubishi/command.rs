
#[derive(Debug, Default)]
pub struct MitsubishiCommand {
    raw: [u8; 18],
    header: [u8; 5],
    on: bool,
    hvac_mode: u8,
    hvac_mode2: u8,
    temp: u8,
    fan: u8,
    clk: u8,
    starttime: u8,
    endtime: u8,
    timer: u8,
    trailer: [u8; 3],
    csum: u8,
}

pub fn chksum(raw: &[u8]) -> u8
{
    (raw.iter().map(|&a| a as u16).fold(0, |s, v| s + v) % 256) as u8
}

impl MitsubishiCommand {

    pub fn new(on: bool) -> Self {
        let mut cmd = MitsubishiCommand::default();
        cmd.on = on;

        cmd
    }

    pub fn pack(&self) -> [u8; 18] {
        let mut dst: [u8; 18] = [35, 203, 38, 1, 0, 32, 72, 3, 192, 124, 0, 0, 0, 0, 16, 64, 56, 68];

        //dst.copy_from_slice(&[35, 203, 38, 1, 0]);
        dst[4] = if self.on {1 << 5} else {0};


        let csum = chksum(&dst[0..17]);
        dst[17] = csum;

        dst
    }

    pub fn unpack(src: [u8; 18]) -> MitsubishiCommand {

        let mut header: [u8; 5] = [0; 5];
        header.copy_from_slice(&src[0..5]);

        let on = (src[5] & 1 << 5) != 0;
        let hvac_mode = src[6];
        let temp = (src[7] & 0xf) + 16;
        let hvac_mode2 = src[8];
        let fan = src[9];
        let clk = src[10];
        let starttime = src[12];
        let endtime = src[11];
        let timer = src[13];
        let mut trailer = [0u8; 3];

        trailer.copy_from_slice(&src[14..17]);
        let csum = src[17];

        MitsubishiCommand {
            raw: src,
            header,
            on,
            hvac_mode,
            temp,
            hvac_mode2,
            fan,
            clk,
            starttime,
            endtime,
            timer,
            trailer,
            csum,
        }
    }

}