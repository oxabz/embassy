//! Example on how to read a Sensirion SHT4x i2c sensor.
//!
//! Connect SDA to P0.03, SCL to P0.04

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Frequency, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Timer;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x52;

bind_interrupts!(struct Irqs {
    SERIAL30 => twim::InterruptHandler<peripherals::SERIAL30>;
});

fn display_pdo(idx: usize, data: &[u8]) {
    let epr = idx > 7;
    let detect = (data[1] >> 7) & 1;
    if detect == 0 {
        return;
    }

    let voltage = data[0];
    let voltage = voltage as u16 * if epr { 200 } else { 100 } as u16;
    let max_current = (data[1] >> 3) & 0x07;
    let typ = (data[1] >> 6) & 1;
    if typ == 0 {
        info!(
            "PDO{} (fixed) voltage = {}mV, max = {}",
            idx, voltage, max_current
        );
    } else {
        let pd_typ = if epr {
            defmt::intern!("AVS")
        } else {
            defmt::intern!("PPS")
        };
        let min_voltage = data[1] & 0x03;
        let min_voltage = match (epr, min_voltage) {
            (_, 0) => intern!("Reserved"),
            (_, 3) => intern!("Unknown"),
            (false, 1) => intern!("3.3V"),
            (false, 2) => intern!("[3.3V - 5V]"),
            (true, 1) => intern!("15V"),
            (true, 2) => intern!("[15V - 20V]"),
            _ => defmt::unreachable!()
        };

        info!("PDO{} ({=istr}) max_voltage = {}mV, min_voltage = {=istr}", idx, pd_typ, voltage, min_voltage);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    Timer::after_millis(10).await;

    info!("Initializing TWI...");
    let mut config = twim::Config::default();
    config.scl_pullup = true;
    config.sda_pullup = true;
    config.frequency = Frequency::K100;
    config.sda_high_drive = false;
    config.scl_high_drive = false;
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);

    let spu_perm = embassy_nrf::pac::SPU20.periph(8).perm().read();
    info!("TWIM spu : ");
    info!(" - SECUREMAPPING : {}", spu_perm.securemapping().to_bits());
    info!(" - DMA : {}", spu_perm.dma().to_bits());
    info!(" - SECATTR : {}", spu_perm.secattr());
    info!(" - DMASEC : {}", spu_perm.dmasec());
    info!(" - LOCK : {}", spu_perm.lock());
    info!(" - PRESENT : {}", spu_perm.present().to_bits() != 0);

    let mut twi = Twim::new(p.SERIAL30, Irqs, p.P0_01, p.P0_02, config, RAM_BUFFER.take());

    info!("Starting measurement...");
    info!("enable  = {:08x}", embassy_nrf::pac::TWIM30.enable().read().0);

    let mut status = [0x01];
    unwrap!(twi.write(ADDRESS, &status).await);
    unwrap!(twi.read(ADDRESS, &mut status).await);
    info!("Status = {}", status[0]);

    let mut opmode = [0x03];
    unwrap!(twi.write(ADDRESS, &opmode).await);
    unwrap!(twi.read(ADDRESS, &mut opmode).await);
    info!("Opmode = {}", opmode[0]);

    let command = [0x11];
    let mut voltage = [0; 2];
    unwrap!(twi.write(ADDRESS, &command).await);
    unwrap!(twi.read(ADDRESS, &mut voltage).await);
    let voltage = (voltage[0] as u16 + ((voltage[1] as u16) << 8)) * 80;
    info!("Current Voltage = {}mV", voltage);

    let ntc25: [_; 2] = u16::to_le_bytes(10000);
    let ntc50: [_; 2] = u16::to_le_bytes(3618);
    let ntc75: [_; 2] = u16::to_le_bytes(1502);
    let ntc100: [_; 2] = u16::to_le_bytes(699);

    let mut ntc25c = [0x0C; 3];
    let mut ntc50c = [0x0D; 3];
    let mut ntc75c = [0x0E; 3];
    let mut ntc100c = [0x0F; 3];

    ntc25c[1..].copy_from_slice(&ntc25);
    ntc50c[1..].copy_from_slice(&ntc50);
    ntc75c[1..].copy_from_slice(&ntc75);
    ntc100c[1..].copy_from_slice(&ntc100);

    unwrap!(twi.write(ADDRESS, &ntc25c).await);
    unwrap!(twi.write(ADDRESS, &ntc50c).await);
    unwrap!(twi.write(ADDRESS, &ntc75c).await);
    unwrap!(twi.write(ADDRESS, &ntc100c).await);

    let command = [0x06];
    let mut temp = [0; 1];
    unwrap!(twi.write(ADDRESS, &command).await);
    unwrap!(twi.read(ADDRESS, &mut temp).await);
    info!("Temperature = {}C", temp[0] + 25);

    let command = [0x20];
    let mut pdo = [0; 26];
    unwrap!(twi.write(ADDRESS, &command).await);
    unwrap!(twi.read(ADDRESS, &mut pdo).await);
    display_pdo(1 , &pdo[0..2]);
    display_pdo(2 , &pdo[2..4]);
    display_pdo(3 , &pdo[4..6]);
    display_pdo(4 , &pdo[6..8]);
    display_pdo(5 , &pdo[8..10]);
    display_pdo(6 , &pdo[10..12]);
    display_pdo(7 , &pdo[12..14]);
    display_pdo(8 , &pdo[14..16]);
    display_pdo(9 , &pdo[16..18]);
    display_pdo(10, &pdo[18..20]);
    display_pdo(11, &pdo[20..22]);
    display_pdo(12, &pdo[22..24]);
    display_pdo(13, &pdo[24..26]);


    let request = [0x31, 0, 0x00];
    unwrap!(twi.write(ADDRESS, &request).await);

    Timer::after_millis(50).await;

    let command = [0x11];
    let mut voltage = [0; 2];
    unwrap!(twi.write(ADDRESS, &command).await);
    unwrap!(twi.read(ADDRESS, &mut voltage).await);
    let voltage = (voltage[0] as u16 + ((voltage[1] as u16) << 8)) * 80;
    info!("Current Voltage = {}mV", voltage);

    loop {}
}
