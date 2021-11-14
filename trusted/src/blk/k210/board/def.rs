/** I/O pins for FPIOA */
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum io {
  /** JTAG TCK */
  JTAG_TCK = 0,
  /** JTAG TDI */
  JTAG_TDI = 1,
  /** JTAG TMS */
  JTAG_TMS = 2,
  /** JTAG TDO */
  JTAG_TDO = 3,
  /** Host RX (from STM32F103C8) */
  ISP_RX = 4,
  /** Host TX (to STM32F103C8) */
  ISP_TX = 5,
  /** WIFI serial TX (from perspective of ESP8285, so our RX) */
  WIFI_TX = 6,
  /** WIFI serial RX (from perspective of ESP8285, so our TX) */
  WIFI_RX = 7,
  /** WIFI enable (to ESP8285) */
  WIFI_EN = 8,
  /** Unused */
  BPSK_P = 9,
  /** Unused */
  BPSK_N = 10,
  /** General purpose I/O pin */
  IO11 = 11,
  /** Blue led (output) */
  LED_B = 12,
  /** Green led (output) */
  LED_G = 13,
  /** Red led (output) */
  LED_R = 14,
  /** Key direction 1 press (input) */
  KEY1 = 15,
  /** Key center press (input) */
  BOOT_KEY0 = 16,
  /** Key direction 2 press (input) */
  KEY2 = 17,
  /** Microphone I2S BCK */
  MIC_BCK = 18,
  /** Microphone I2S WS */
  MIC_WS = 19,
  /** Microphone I2S DAT3 */
  MIC_DAT3 = 20,
  /** Microphone I2S DAT2 */
  MIC_DAT2 = 21,
  /** Microphone I2S DAT1 */
  MIC_DAT1 = 22,
  /** Microphone I2S DAT0 */
  MIC_DAT0 = 23,
  /** Microphone LED DAT */
  MIC_LED_DAT = 24,
  /** Microphone LED CLK */
  MIC_LED_CLK = 25,
  /** SDCARD SPI MISO */
  SPI0_MISO = 26,
  /** SDCARD SPI SCLK */
  SPI0_SCLK = 27,
  /** SDCARD SPI MOSI */
  SPI0_MOSI = 28,
  /** SDCARD SPI CS */
  SPI0_CS0 = 29,
  /** I2C bus 1 SCLK (NS2009, MSA300) */
  I2C1_SCL = 30,
  /** I2C bus 2 SDA (NS2009, MSA300) */
  I2C1_SDA = 31,
  /** General purpose I/O pin */
  IO32 = 32,
  /** DAC I2S WS */
  I2S_WS = 33,
  /** DAC I2S DA */
  I2S_DA = 34,
  /** DAC I2S BCK */
  I2S_BCK = 35,
  /** LCD chip select (output) */
  LCD_CS = 36,
  /** LCD reset (output) */
  LCD_RST = 37,
  /** LCD Data/Command */
  LCD_DC = 38,
  /** LCD SPI SCLK */
  LCD_WR = 39,
  /** Camera DVP SDA */
  DVP_SDA = 40,
  /** Camera DVP SCL */
  DVP_SCL = 41,
  /** Camera DVP RST */
  DVP_RST = 42,
  /** Camera DVP VSYNC */
  DVP_VSYNC = 43,
  /** Camera DVP PWDN */
  DVP_PWDN = 44,
  /** Camera DVP HSYNC */
  DVP_HSYNC = 45,
  /** Camera DVP XCLK */
  DVP_XCLK = 46,
  /** Camera DVP PCLK */
  DVP_PCLK = 47,
}

impl From<io> for usize {
  fn from(io: io) -> Self {
    io as usize
  }
}
