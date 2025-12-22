//! Device identification utilities for USB peripherals.
//!
//! Provides vendor name, device model, and HID device type resolution
//! based on VID/PID and HID Usage information.

/// Resolves vendor name from USB Vendor ID.
///
/// Returns None for unrecognized vendors.
#[inline]
pub fn get_vendor_name(vid: u16) -> Option<&'static str> {
    match vid {
        // Major console manufacturers
        0x045E => Some("Microsoft"),
        0x054C => Some("Sony"),
        0x057E => Some("Nintendo"),

        // PC peripheral manufacturers
        0x046D => Some("Logitech"),
        0x1532 => Some("Razer"),
        0x1038 => Some("SteelSeries"),
        0x0B05 => Some("ASUS"),
        0x1B1C => Some("Corsair"),
        0x1462 => Some("MSI"),
        0x0C45 => Some("Microdia"),
        0x1689 => Some("Razer"),
        0x413C => Some("Dell"),
        0x05AC => Some("Apple"),
        0x17EF => Some("Lenovo"),
        0x03F0 => Some("HP"),
        0x0B97 => Some("O2 Micro"),
        0x174C => Some("ASMedia"),
        0x152D => Some("JMicron"),
        0x0BDA => Some("Realtek"),
        0x8087 => Some("Intel"),
        0x8086 => Some("Intel"),
        0x10DE => Some("Authenex"),

        // Gaming controller manufacturers
        0x0738 => Some("Mad Catz"),
        0x0E6F => Some("PDP"),
        0x0F0D => Some("HORI"),
        0x2563 => Some("8BitDo"),
        0x2DC8 => Some("8BitDo"),
        0x0079 => Some("DragonRise"),
        0x0810 => Some("Personal Communication Systems"),
        0x24C6 => Some("PowerA"),
        0x20D6 => Some("PowerA"),
        0x0E8F => Some("GameSir"),
        0x0F30 => Some("Qanba"),
        0x1BAD => Some("Harmonix"),
        0x1430 => Some("RedOctane"),
        0x12BA => Some("Licensed by Sony"),
        0x11C0 => Some("Betop"),
        0x0B38 => Some("Gear Head"),
        0x146B => Some("BigBen"),
        0x1A34 => Some("ACRUX"),
        0x2C22 => Some("Qanba"),
        0x0C12 => Some("Zeroplus"),
        0x045B => Some("Hitachi"),
        0x0E8D => Some("MediaTek"),

        // Additional gaming brands
        0x1949 => Some("Lab126"),
        0x0CA3 => Some("Redragon"),
        0x3285 => Some("Nacon"),
        0x1EA7 => Some("SHARKOON"),
        0x0D8C => Some("C-Media"),
        0x1CCF => Some("Pronove Solutions"),
        0x0F39 => Some("TG3 Electronics"),
        0x20D1 => Some("ASUS"),
        0x0B43 => Some("Play.com"),
        0x045D => Some("Nortel Networks"),
        0x0563 => Some("Immersion"),
        0x06A7 => Some("MicroStore"),

        // Valve and Steam
        0x28DE => Some("Valve"),

        // Flight sim and specialized controllers
        0x044F => Some("ThrustMaster"),
        0x06A3 => Some("Saitek"),
        0x231D => Some("Plasma Cloud"),
        0x0483 => Some("STMicroelectronics"),
        0x16C0 => Some("Van Ooijen Technische Informatica"),
        0x1B4F => Some("SparkFun Electronics"),
        0x239A => Some("Adafruit"),
        0x2341 => Some("Arduino"),
        0x1050 => Some("Yubico"),
        0x20A0 => Some("Clay Logic"),

        // VR Controllers
        0x0BB4 => Some("HTC"),
        0x2833 => Some("Oculus"),
        0x0F4C => Some("WorldWide Cable Opto"),

        // Mobile and tablet manufacturers
        0x18D1 => Some("Google"),
        0x2717 => Some("Xiaomi"),
        0x2A45 => Some("Meizu"),
        0x2A70 => Some("OnePlus"),
        0x19D2 => Some("ZTE"),
        0x12D1 => Some("Huawei"),
        0x0BB0 => Some("Concord Camera"),
        0x04E8 => Some("Samsung"),
        0x22B8 => Some("Motorola"),
        0x0FCE => Some("Sony Ericsson"),
        0x0955 => Some("NVIDIA"),
        0x2207 => Some("Rockchip"),
        0x1F3A => Some("Allwinner"),
        0x2A03 => Some("dog hunter AG"),

        // Audio equipment manufacturers
        0x0763 => Some("M-Audio"),
        0x0582 => Some("Roland"),
        0x0944 => Some("KORG"),
        0x0499 => Some("Yamaha"),
        0x1235 => Some("Focusrite-Novation"),
        0x17CC => Some("Native Instruments"),
        0x041E => Some("Creative Technology"),
        0x047F => Some("Plantronics"),
        0x046E => Some("Behavior Tech Computer"),
        0x0A12 => Some("Cambridge Silicon Radio"),
        0x05A7 => Some("Bose"),
        0x0D8E => Some("Global Sun Technology"),

        // Camera manufacturers
        0x04A9 => Some("Canon"),
        0x04B0 => Some("Nikon"),
        0x04CB => Some("Fuji Photo Film"),
        0x056A => Some("Wacom"),
        0x056C => Some("eTEK Labs"),
        0x0693 => Some("Hagiwara Sys-Com"),
        0x04DA => Some("Panasonic"),

        // Network equipment
        0x0846 => Some("NetGear"),
        0x2001 => Some("D-Link"),
        0x0CF3 => Some("Qualcomm Atheros"),
        0x148F => Some("Ralink Technology"),
        0x2357 => Some("TP-Link"),
        0x0586 => Some("ZyXEL"),
        0x13B1 => Some("Linksys"),

        // PC peripheral brands
        0x1E7D => Some("ROCCAT"),
        0x2516 => Some("Cooler Master"),
        0x1E71 => Some("NZXT"),
        0x0951 => Some("HyperX"),
        0x264A => Some("Thermaltake"),
        0x145F => Some("Trust"),
        0x09DA => Some("A4Tech"),
        0x11C9 => Some("Nacon"),

        _ => None,
    }
}

/// Resolves specific device model from USB VID:PID pair.
///
/// Returns device model name for known gaming peripherals.
#[inline]
pub fn get_device_model(vid: u16, pid: u16) -> Option<&'static str> {
    match (vid, pid) {
        // Microsoft Xbox Controllers
        (0x045E, 0x0202) => Some("Xbox Controller"),
        (0x045E, 0x0285) => Some("Xbox Controller S"),
        (0x045E, 0x0289) => Some("Xbox Controller S"),
        (0x045E, 0x028E) => Some("Xbox 360 Controller"),
        (0x045E, 0x028F) => Some("Xbox 360 Wireless Controller"),
        (0x045E, 0x0291) => Some("Xbox 360 Wireless Receiver"),
        (0x045E, 0x02D1) => Some("Xbox One Controller"),
        (0x045E, 0x02DD) => Some("Xbox One Controller (FW 2015)"),
        (0x045E, 0x02E0) => Some("Xbox One Wireless Controller"),
        (0x045E, 0x02E3) => Some("Xbox One Elite Controller"),
        (0x045E, 0x02EA) => Some("Xbox One Controller"),
        (0x045E, 0x02FD) => Some("Xbox One S Controller (BT)"),
        (0x045E, 0x0719) => Some("Xbox 360 Wireless Adapter"),
        (0x045E, 0x0B00) => Some("Xbox Elite Series 2 Controller"),
        (0x045E, 0x0B12) => Some("Xbox Series X|S Controller"),
        (0x045E, 0x0B13) => Some("Xbox Wireless Controller"),

        // Microsoft SideWinder Series
        (0x045E, 0x0007) => Some("SideWinder Game Pad"),
        (0x045E, 0x0008) => Some("SideWinder Precision Pro"),
        (0x045E, 0x001A) => Some("SideWinder Precision Racing Wheel"),
        (0x045E, 0x001B) => Some("SideWinder Force Feedback 2"),
        (0x045E, 0x0026) => Some("SideWinder GamePad Pro"),
        (0x045E, 0x0027) => Some("SideWinder PnP GamePad"),
        (0x045E, 0x003C) => Some("SideWinder Joystick"),
        (0x045E, 0x0034) => Some("SideWinder Force Feedback Wheel"),

        // Sony PlayStation Controllers
        (0x054C, 0x0268) => Some("PlayStation 3 Controller"),
        (0x054C, 0x042F) => Some("PlayStation Move Navigation Controller"),
        (0x054C, 0x03D5) => Some("PlayStation Move Motion Controller"),
        (0x054C, 0x05C4) => Some("DualShock 4 [CUH-ZCT1x]"),
        (0x054C, 0x09CC) => Some("DualShock 4 [CUH-ZCT2x]"),
        (0x054C, 0x0BA0) => Some("DualShock 4 Wireless Adapter"),
        (0x054C, 0x0CDA) => Some("PlayStation Classic Controller"),
        (0x054C, 0x0CE6) => Some("DualSense Wireless Controller"),
        (0x054C, 0x0DF2) => Some("DualSense Edge Wireless Controller"),
        (0x054C, 0x02EA) => Some("PlayStation 3 Memory Card Adapter"),

        // Nintendo Controllers
        (0x057E, 0x2000) => Some("Nintendo Switch"),
        (0x057E, 0x2006) => Some("Joy-Con (L)"),
        (0x057E, 0x2007) => Some("Joy-Con (R)"),
        (0x057E, 0x2009) => Some("Switch Pro Controller"),
        (0x057E, 0x200E) => Some("Joy-Con Charging Grip"),
        (0x057E, 0x0306) => Some("Wii Remote Controller"),
        (0x057E, 0x0337) => Some("Wii U GameCube Controller Adapter"),
        (0x057E, 0x0341) => Some("Wii U Pro Controller Host"),

        // 8BitDo Controllers
        (0x2DC8, 0x5006) => Some("8BitDo M30 Bluetooth"),
        (0x2DC8, 0x6000) => Some("8BitDo SF30 Pro"),
        (0x2DC8, 0x6001) => Some("8BitDo SN30/SF30 Pro"),
        (0x2DC8, 0xAB11) => Some("8BitDo F30"),
        (0x2DC8, 0xAB12) => Some("8BitDo N30"),
        (0x2DC8, 0xAB20) => Some("8BitDo SN30/SF30"),
        (0x2DC8, 0xAB21) => Some("8BitDo SF30"),

        // Logitech Controllers
        (0x046D, 0xC216) => Some("Logitech F310 (DirectInput)"),
        (0x046D, 0xC218) => Some("Logitech F510 (DirectInput)"),
        (0x046D, 0xC219) => Some("Logitech F710 (DirectInput)"),
        (0x046D, 0xC21D) => Some("Logitech F310 (XInput)"),
        (0x046D, 0xC21E) => Some("Logitech F510 (XInput)"),
        (0x046D, 0xC21F) => Some("Logitech F710 (XInput)"),
        (0x046D, 0xC21A) => Some("Logitech Precision Gamepad"),
        (0x046D, 0xCA84) => Some("Logitech Cordless Controller for Xbox"),
        (0x046D, 0xCA88) => Some("Logitech Thunderpad for Xbox"),

        // Razer Controllers
        (0x1532, 0x0A00) => Some("Razer Atrox for Xbox One"),
        (0x1532, 0x0A03) => Some("Razer Wildcat"),
        (0x1532, 0x0A14) => Some("Razer Raiju PS4"),
        (0x1532, 0x1000) => Some("Razer Panthera"),
        (0x1532, 0x1007) => Some("Razer Raiju 2"),
        (0x1532, 0x1008) => Some("Razer Panthera Evo"),
        (0x1532, 0x1009) => Some("Razer Raiju Ultimate"),

        // HORI Controllers
        (0x0F0D, 0x0011) => Some("HORI Fighting Commander"),
        (0x0F0D, 0x0063) => Some("HORI Real Arcade Pro Hayabusa"),
        (0x0F0D, 0x0078) => Some("HORI Real Arcade Pro V Kai"),
        (0x0F0D, 0x0092) => Some("HORI Pokken Tournament DX Pro Pad"),
        (0x0F0D, 0x00AA) => Some("HORI Real Arcade Pro V Kai"),
        (0x0F0D, 0x00C1) => Some("HORI Battle Pad for Switch"),
        (0x0F0D, 0x000C) => Some("HORI Horipad EX Turbo"),
        (0x0F0D, 0x000D) => Some("HORI Fighting Stick EX2"),

        // Mad Catz Controllers
        (0x0738, 0x4716) => Some("Mad Catz Wired Xbox 360 Controller"),
        (0x0738, 0x4718) => Some("Mad Catz Street Fighter IV FightStick SE"),
        (0x0738, 0x4726) => Some("Mad Catz Xbox 360 Controller"),
        (0x0738, 0x4728) => Some("Mad Catz Street Fighter IV FightPad"),
        (0x0738, 0x4736) => Some("Mad Catz MicroCon for Xbox 360"),
        (0x0738, 0x4738) => Some("Mad Catz Street Fighter IV Wired Controller"),
        (0x0738, 0x4740) => Some("Mad Catz Beat Pad"),
        (0x0738, 0x4A01) => Some("Mad Catz FightStick TE 2"),
        (0x0738, 0xB738) => Some("Mad Catz Marvel VS Capcom 2 TE"),
        (0x0738, 0xF738) => Some("Mad Catz Super Street Fighter IV TE S"),

        // PowerA Controllers
        (0x24C6, 0x5300) => Some("PowerA Mini ProEX Controller"),
        (0x24C6, 0x530A) => Some("PowerA ProEX Controller"),
        (0x24C6, 0x541A) => Some("PowerA Xbox One Mini Controller"),
        (0x24C6, 0x542A) => Some("PowerA Spectra"),
        (0x24C6, 0x543A) => Some("PowerA Xbox One Wired Controller"),
        (0x24C6, 0x561A) => Some("PowerA Fusion Controller"),
        (0x24C6, 0x5B02) => Some("PowerA Fusion Pro Controller"),

        // PDP Controllers
        (0x0E6F, 0x0105) => Some("PDP Disney High School Musical 3 Dance Pad"),
        (0x0E6F, 0x0113) => Some("PDP Afterglow AX.1"),
        (0x0E6F, 0x011F) => Some("PDP Rock Candy Wired Controller"),
        (0x0E6F, 0x0139) => Some("PDP Afterglow Prismatic"),
        (0x0E6F, 0x013A) => Some("PDP Xbox One Controller"),
        (0x0E6F, 0x0146) => Some("PDP Rock Candy Wired Controller"),
        (0x0E6F, 0x0147) => Some("PDP Marvel Controller"),
        (0x0E6F, 0x015C) => Some("PDP Arcade Stick"),
        (0x0E6F, 0x0161) => Some("PDP Camo Wired Controller"),
        (0x0E6F, 0x0162) => Some("PDP Xbox One Wired Controller"),
        (0x0E6F, 0x0164) => Some("PDP Battlefield 1 Wired Controller"),
        (0x0E6F, 0x0165) => Some("PDP Titanfall 2 Wired Controller"),
        (0x0E6F, 0x02A4) => Some("PDP Wired Controller"),

        // Steam/Valve Controllers
        (0x28DE, 0x1102) => Some("Steam Controller"),
        (0x28DE, 0x1142) => Some("Steam Controller (BLE)"),
        (0x28DE, 0x1205) => Some("Steam Deck"),

        // Nacon Controllers
        (0x3285, 0x0003) => Some("Nacon GC-400ES"),
        (0x3285, 0x0010) => Some("Nacon Revolution Pro Controller"),
        (0x3285, 0x0607) => Some("Nacon GC-100"),

        // ThrustMaster
        (0x044F, 0x0402) => Some("ThrustMaster HOTAS Warthog"),
        (0x044F, 0xB10A) => Some("ThrustMaster T.16000M"),
        (0x044F, 0xB679) => Some("ThrustMaster T-Flight HOTAS X"),

        // DragonRise Inc
        (0x0079, 0x0006) => Some("DragonRise PC TWIN SHOCK"),
        (0x0079, 0x0011) => Some("DragonRise Gamepad"),
        (0x0079, 0x1800) => Some("Mayflash Wii U Pro Adapter"),
        (0x0079, 0x181B) => Some("Venom Arcade Joystick"),
        (0x0079, 0x1843) => Some("Mayflash GameCube Adapter"),
        (0x0079, 0x1844) => Some("Mayflash GameCube Controller"),

        // Logitech Gaming Mice & Keyboards
        (0x046D, 0xC062) => Some("Logitech Gaming Mouse G500"),
        (0x046D, 0xC065) => Some("Logitech G19 Gaming Keyboard"),
        (0x046D, 0xC07D) => Some("Logitech G502 Proteus Core"),
        (0x046D, 0xC07E) => Some("Logitech G502 Proteus Spectrum"),
        (0x046D, 0xC24E) => Some("Logitech G700s Gaming Mouse"),
        (0x046D, 0xC332) => Some("Logitech G502 HERO"),
        (0x046D, 0xC531) => Some("Logitech G PRO Gaming Mouse"),
        (0x046D, 0xC539) => Some("Logitech G PRO X SUPERLIGHT"),

        // Razer Gaming Peripherals
        (0x1532, 0x0003) => Some("Razer Krait"),
        (0x1532, 0x0007) => Some("Razer DeathAdder"),
        (0x1532, 0x0010) => Some("Razer Copperhead"),
        (0x1532, 0x0013) => Some("Razer Orochi"),
        (0x1532, 0x0016) => Some("Razer DeathAdder 3.5G"),
        (0x1532, 0x0017) => Some("Razer Lachesis"),
        (0x1532, 0x001C) => Some("Razer Mamba"),
        (0x1532, 0x0024) => Some("Razer Mamba Elite"),
        (0x1532, 0x0029) => Some("Razer DeathAdder Black Edition"),
        (0x1532, 0x002E) => Some("Razer Naga"),
        (0x1532, 0x0033) => Some("Razer BlackWidow"),
        (0x1532, 0x0037) => Some("Razer DeathAdder 2013"),
        (0x1532, 0x0043) => Some("Razer DeathAdder Chroma"),
        (0x1532, 0x0053) => Some("Razer DeathAdder Elite"),
        (0x1532, 0x005C) => Some("Razer Viper Ultimate"),
        (0x1532, 0x006A) => Some("Razer BlackWidow V3"),
        (0x1532, 0x0078) => Some("Razer Viper"),
        (0x1532, 0x0084) => Some("Razer Basilisk V2"),
        (0x1532, 0x0216) => Some("Razer BlackWidow Ultimate 2013"),
        (0x1532, 0x0221) => Some("Razer BlackWidow Chroma"),
        (0x1532, 0x0227) => Some("Razer BlackWidow X Chroma"),
        (0x1532, 0x0233) => Some("Razer Huntsman Elite"),
        (0x1532, 0x0241) => Some("Razer BlackWidow V3 Pro"),
        (0x1532, 0x0510) => Some("Razer Kraken 7.1"),
        (0x1532, 0x0520) => Some("Razer Kraken 7.1 Chroma"),
        (0x1532, 0x0527) => Some("Razer Kraken Ultimate"),

        // Corsair Gaming Peripherals
        (0x1B1C, 0x0A60) => Some("Corsair Vengeance K60"),
        (0x1B1C, 0x1B04) => Some("Corsair Raptor K50"),
        (0x1B1C, 0x1B07) => Some("Corsair Vengeance K65"),
        (0x1B1C, 0x1B08) => Some("Corsair Vengeance K95"),
        (0x1B1C, 0x1B09) => Some("Corsair Vengeance K70R"),
        (0x1B1C, 0x1B11) => Some("Corsair K95 RGB Mechanical"),
        (0x1B1C, 0x1B13) => Some("Corsair Vengeance K70 RGB"),
        (0x1B1C, 0x1B20) => Some("Corsair STRAFE RGB"),
        (0x1B1C, 0x1B2D) => Some("Corsair K95 RGB Platinum"),
        (0x1B1C, 0x1B2E) => Some("Corsair M65 Pro RGB"),
        (0x1B1C, 0x1B2F) => Some("Corsair Sabre RGB"),
        (0x1B1C, 0x1B3D) => Some("Corsair K55 RGB"),
        (0x1B1C, 0x1B5E) => Some("Corsair Harpoon Wireless"),

        // SteelSeries Gaming Peripherals
        (0x1038, 0x0100) => Some("SteelSeries Ideazon Zboard"),
        (0x1038, 0x1260) => Some("SteelSeries Arctis 7 Wireless"),
        (0x1038, 0x1361) => Some("SteelSeries Ideazon Sensei"),
        (0x1038, 0x1410) => Some("SteelSeries SRW-S1 Racing Wheel"),
        (0x1038, 0x1720) => Some("SteelSeries Gaming Mouse"),

        // ROCCAT Gaming Peripherals
        (0x1E7D, 0x2C24) => Some("ROCCAT Pyra Wired"),
        (0x1E7D, 0x2C2E) => Some("ROCCAT Lua"),
        (0x1E7D, 0x2C38) => Some("ROCCAT Kiro"),
        (0x1E7D, 0x2CED) => Some("ROCCAT Kone"),
        (0x1E7D, 0x2CEE) => Some("ROCCAT Kova 2016 Gray"),
        (0x1E7D, 0x2CEF) => Some("ROCCAT Kova 2016 White"),
        (0x1E7D, 0x2CF0) => Some("ROCCAT Kova 2016 Black"),
        (0x1E7D, 0x2CF6) => Some("ROCCAT Pyra Wireless"),
        (0x1E7D, 0x2D50) => Some("ROCCAT Kova[+]"),
        (0x1E7D, 0x2D51) => Some("ROCCAT Kone[+]"),
        (0x1E7D, 0x2D5A) => Some("ROCCAT Savu"),
        (0x1E7D, 0x2DB4) => Some("ROCCAT Kone Pure Optical"),
        (0x1E7D, 0x2E27) => Some("ROCCAT Kone AIMO"),
        (0x1E7D, 0x2E4A) => Some("ROCCAT Tyon Black"),
        (0x1E7D, 0x2E7C) => Some("ROCCAT Nyth Black"),
        (0x1E7D, 0x2F76) => Some("ROCCAT Sova"),
        (0x1E7D, 0x2FA8) => Some("ROCCAT Suora"),
        (0x1E7D, 0x2FC6) => Some("ROCCAT Skeltr"),
        (0x1E7D, 0x30D4) => Some("ROCCAT Arvo"),
        (0x1E7D, 0x3138) => Some("ROCCAT Ryos MK"),
        (0x1E7D, 0x319C) => Some("ROCCAT Isku"),

        // Cooler Master Gaming Peripherals
        (0x2516, 0x0003) => Some("Cooler Master Storm Xornet"),
        (0x2516, 0x0004) => Some("Cooler Master Storm QuickFire Rapid"),
        (0x2516, 0x0006) => Some("Cooler Master Storm Recon"),
        (0x2516, 0x0007) => Some("Cooler Master Storm Sentinel Advance II"),
        (0x2516, 0x0009) => Some("Cooler Master Storm Quick Fire PRO"),
        (0x2516, 0x0015) => Some("Cooler Master Storm QuickFire Pro/Ultimate"),
        (0x2516, 0x0017) => Some("Cooler Master Storm Quick Fire Stealth"),
        (0x2516, 0x001A) => Some("Cooler Master Storm Quick Fire XT"),
        (0x2516, 0x0020) => Some("Cooler Master QuickFire Rapid-i"),
        (0x2516, 0x0027) => Some("Cooler Master CM Storm Novatouch TKL"),
        (0x2516, 0x002D) => Some("Cooler Master Alcor"),
        (0x2516, 0x0042) => Some("Cooler Master Masterkeys Lite L RGB"),
        (0x2516, 0x0046) => Some("Cooler Master Masterkeys PRO L"),
        (0x2516, 0x0047) => Some("Cooler Master MasterKeys Pro L"),
        (0x2516, 0x0055) => Some("Cooler Master MasterKeys L"),

        // NZXT Gaming Peripherals
        (0x1E71, 0x0001) => Some("NZXT Avatar Optical Mouse"),
        (0x1E71, 0x170E) => Some("NZXT Kraken X"),
        (0x1E71, 0x1711) => Some("NZXT Grid+ V3"),
        (0x1E71, 0x1714) => Some("NZXT Smart Device"),
        (0x1E71, 0x1715) => Some("NZXT Kraken M22"),
        (0x1E71, 0x2006) => Some("NZXT Smart Device V2"),

        // HyperX Gaming Peripherals
        (0x0951, 0x16A4) => Some("HyperX Cloud Flight Wireless"),
        (0x0951, 0x16C4) => Some("HyperX Cloud Flight"),
        (0x0951, 0x16D2) => Some("HyperX Alloy FPS Pro"),
        (0x0951, 0x16DF) => Some("HyperX QuadCast"),
        (0x0951, 0x16E4) => Some("HyperX Pulsefire Raid"),
        (0x0951, 0x16B3) => Some("HyperX Savage"),

        // Thermaltake Gaming Peripherals
        (0x264A, 0x1004) => Some("Thermaltake Ventus"),

        // Sharkoon Gaming Peripherals
        (0x1EA7, 0x0030) => Some("Sharkoon Trust GXT 158 Gaming Mouse"),
        (0x1EA7, 0x1002) => Some("Sharkoon Vintorez Gaming Mouse"),
        (0x1EA7, 0x2007) => Some("Sharkoon SHARK ZONE K30"),

        // GameSir Controllers
        (0x0E8F, 0x3013) => Some("GameSir G3w"),
        (0x0E8F, 0x0003) => Some("GameSir G7"),

        // Qanba Arcade Sticks
        (0x0F30, 0x0111) => Some("Qanba Q4RAF"),
        (0x0F30, 0x00F8) => Some("Qanba Drone"),
        (0x0F30, 0x1100) => Some("Qanba Obsidian"),

        // Nacon Extended
        (0x11C9, 0x5500) => Some("Nacon Daija Arcade Stick"),

        // Saitek Flight Controllers
        (0x06A3, 0x0001) => Some("Saitek Joystick"),
        (0x06A3, 0x0006) => Some("Saitek Cyborg Gold Joystick"),
        (0x06A3, 0x0201) => Some("Saitek Adrenalin Gamepad"),
        (0x06A3, 0x0241) => Some("Saitek Xbox Adrenalin Gamepad"),
        (0x06A3, 0x0255) => Some("Saitek X52 Flight Controller"),
        (0x06A3, 0x075C) => Some("Saitek X52 Flight Controller"),
        (0x06A3, 0x0762) => Some("Saitek Saitek X52 Pro Flight Controller"),
        (0x06A3, 0x0B4E) => Some("Saitek Pro Flight Cessna Yoke"),
        (0x06A3, 0x0C2D) => Some("Saitek Pro Flight Multi Panel"),
        (0x06A3, 0x0D05) => Some("Saitek Pro Flight Radio Panel"),
        (0x06A3, 0x0D06) => Some("Saitek Pro Flight Switch Panel"),
        (0x06A3, 0x0D67) => Some("Saitek Pro Flight Switch Panel"),

        // Trust Gaming Peripherals
        (0x145F, 0x01C2) => Some("Trust GXT 152 Illuminated Gaming Mouse"),
        (0x145F, 0x0212) => Some("Trust Gaming Mouse"),

        // A4Tech Peripherals
        (0x09DA, 0x9066) => Some("A4Tech Bloody V8 Gaming Mouse"),
        (0x09DA, 0x9090) => Some("A4Tech X-718BK Keyboard"),
        (0x09DA, 0x90C0) => Some("A4Tech X7 G800V Keyboard"),

        // Redragon Gaming Peripherals
        (0x0CA3, 0x0021) => Some("Redragon Gaming Keyboard"),
        (0x0CA3, 0x0023) => Some("Redragon Gaming Mouse"),

        _ => None,
    }
}

/// Maps HID Usage Page and Usage to human-readable device type.
#[inline]
pub fn get_hid_device_type(usage_page: u16, usage: u16) -> &'static str {
    match (usage_page, usage) {
        (0x0001, 0x0004) => "Joystick",
        (0x0001, 0x0005) => "Gamepad",
        (0x0001, 0x0006) => "Keyboard",
        (0x0001, 0x0002) => "Mouse",
        (0x0001, 0x0008) => "Multi-axis Controller",
        _ => "HID Device",
    }
}
