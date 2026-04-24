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

        // Keyboard / keypad specialists
        0x3434 => Some("Keychron"),
        0x320F => Some("Akko"),
        0x258A => Some("SINO WEALTH"),
        0x04D9 => Some("Holtek"),
        0x05F3 => Some("Kinesis"),
        0x2B24 => Some("Keebio"),
        0x3297 => Some("ZSA Technology Labs"),
        0x17F6 => Some("Unicomp"),
        0x31E3 => Some("Wooting"),
        0x342D => Some("Varmilo"),
        0x3151 => Some("NuPhy"),
        0x320C => Some("HyperX (HP)"),

        // Controller brands and clone makers
        0x32C2 => Some("Flydigi"),
        0x3537 => Some("GameSir"),
        0x33DF => Some("BigBigWon"),
        0x2993 => Some("Rainbow Electronics"),
        0x23B5 => Some("EasySMX"),
        0x9886 => Some("Astro Gaming"),
        0x2E24 => Some("Hyperkin"),
        0x2E95 => Some("SCUF Gaming"),
        0x3767 => Some("Fanatec"),
        0x0583 => Some("Padix (Rockfire)"),
        0x068E => Some("CH Products"),
        0x06F8 => Some("Guillemot"),
        0x10F5 => Some("Turtle Beach"),
        0x124B => Some("Nyko (Honey Bee)"),
        0x12AB => Some("Honey Bee Electronic"),
        0x1451 => Some("Force Dimension"),
        0x1BAE => Some("Vuzix"),
        0x187C => Some("Alienware"),
        0x21A4 => Some("Electronic Arts"),
        0x24AE => Some("Rapoo"),
        0x20BC => Some("ShanWan (Shenzhen)"),
        0x25F0 => Some("ShanWan"),
        0x0E4C => Some("Radica Games"),
        0x104F => Some("WB Electronics"),
        0x19FA => Some("Gampaq"),
        0x1690 => Some("Askey Computer"),
        0xD208 => Some("Ultimarc"),
        0xD209 => Some("Ultimarc"),
        0x046A => Some("CHERRY"),
        0x047D => Some("Kensington"),

        // Drawing tablets and pen displays
        0x28BD => Some("XP-PEN"),
        0x256C => Some("Huion"),
        0x172F => Some("Waltop"),
        0x0B57 => Some("Bosto Tablet"),

        // Streaming and capture hardware
        0x0FD9 => Some("Elgato"),
        0x1EDB => Some("Blackmagic Design"),
        0x2040 => Some("Hauppauge"),

        // Chipset and silicon vendors commonly seen in HID reports.
        0x04F2 => Some("Chicony Electronics"),
        0x0458 => Some("KYE Systems"),
        0x093A => Some("Pixart Imaging"),
        0x04CA => Some("Lite-On"),
        0x04F3 => Some("Elan Microelectronics"),
        0x06CB => Some("Synaptics"),
        0x10C4 => Some("Silicon Labs"),
        0x0403 => Some("FTDI"),
        0x1A86 => Some("QinHeng Electronics"),
        0x1BCF => Some("Sunplus Innovation"),
        0x03EB => Some("Atmel"),
        0x0416 => Some("Winbond Electronics"),
        0x1366 => Some("SEGGER"),
        0x1915 => Some("Nordic Semiconductor"),
        0x0A5C => Some("Broadcom"),
        0x0D62 => Some("Darfon Electronics"),

        // Storage and memory
        0x0BC2 => Some("Seagate"),
        0x1058 => Some("Western Digital"),
        0x0781 => Some("SanDisk"),
        0x152E => Some("Kingston"),
        0x0930 => Some("Toshiba"),

        // Mobile and tablet manufacturers, extended list
        0x22D9 => Some("OPPO"),
        0x2916 => Some("Vivo"),
        0x2915 => Some("Nothing Technology"),
        0x1C7A => Some("Nothing"),

        // Console and handheld peripherals
        0x273F => Some("Hyperkin"),
        0x1CF1 => Some("Dresden Elektronik"),

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
        (0x045E, 0x0B05) => Some("Xbox Elite Series 2 Core Controller"),
        (0x045E, 0x0B0A) => Some("Xbox Adaptive Controller"),
        (0x045E, 0x0B0C) => Some("Xbox Wireless Adapter for Windows"),
        (0x045E, 0x0B12) => Some("Xbox Series X|S Controller"),
        (0x045E, 0x0B13) => Some("Xbox Wireless Controller"),
        (0x045E, 0x0B20) => Some("Xbox Wireless Controller (BLE)"),
        (0x045E, 0x0B22) => Some("Xbox Wireless Controller (Model 1914)"),
        (0x045E, 0x0B3A) => Some("Xbox Wireless Headset"),

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
        (0x054C, 0x0154) => Some("PlayStation Eyetoy Audio"),
        (0x054C, 0x02EA) => Some("PlayStation 3 Memory Card Adapter"),

        // Nintendo Controllers
        (0x057E, 0x2000) => Some("Nintendo Switch"),
        (0x057E, 0x2006) => Some("Joy-Con (L)"),
        (0x057E, 0x2007) => Some("Joy-Con (R)"),
        (0x057E, 0x2009) => Some("Switch Pro Controller"),
        (0x057E, 0x200E) => Some("Joy-Con Charging Grip"),
        (0x057E, 0x0300) => Some("USB-EXI GameCube Adapter (GCP-2000)"),
        (0x057E, 0x0304) => Some("RVT-H Reader"),
        (0x057E, 0x0306) => Some("Wii Remote Controller"),
        (0x057E, 0x0337) => Some("Wii U GameCube Controller Adapter"),
        (0x057E, 0x0341) => Some("Wii U Pro Controller Host"),

        // 8BitDo Controllers
        (0x2DC8, 0x2867) => Some("8BitDo Ultimate Controller (2.4G)"),
        (0x2DC8, 0x3106) => Some("8BitDo Pro 2 Wired"),
        (0x2DC8, 0x310A) => Some("8BitDo Pro 2 (Bluetooth)"),
        (0x2DC8, 0x5006) => Some("8BitDo M30 Bluetooth"),
        (0x2DC8, 0x6000) => Some("8BitDo SF30 Pro"),
        (0x2DC8, 0x6001) => Some("8BitDo SN30/SF30 Pro"),
        (0x2DC8, 0x6100) => Some("8BitDo SN30 Pro+"),
        (0x2DC8, 0x9001) => Some("8BitDo Zero"),
        (0x2DC8, 0x9015) => Some("8BitDo M30 Wireless"),
        (0x2DC8, 0x9018) => Some("8BitDo Zero 2"),
        (0x2DC8, 0xAB11) => Some("8BitDo F30"),
        (0x2DC8, 0xAB12) => Some("8BitDo N30"),
        (0x2DC8, 0xAB20) => Some("8BitDo SN30/SF30"),
        (0x2DC8, 0xAB21) => Some("8BitDo SF30"),
        (0x2DC8, 0xAB30) => Some("8BitDo NES30 Pro"),

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
        (0x1532, 0x100A) => Some("Razer Raiju 2 Tournament Edition (BT)"),
        (0x1532, 0x1004) => Some("Razer Raiju Ultimate Wired"),
        (0x1532, 0x1016) => Some("Razer Wolverine V2"),
        (0x1532, 0x1019) => Some("Razer Wolverine V2 Chroma"),
        (0x1532, 0x1020) => Some("Razer Kitsune"),

        // HORI Controllers
        (0x0F0D, 0x000A) => Some("HORI Dead or Alive 4 FightStick"),
        (0x0F0D, 0x000C) => Some("HORI Horipad EX Turbo for Xbox 360"),
        (0x0F0D, 0x000D) => Some("HORI Fighting Stick EX2 for Xbox 360"),
        (0x0F0D, 0x0011) => Some("HORI Fighting Commander"),
        (0x0F0D, 0x0016) => Some("HORI Real Arcade Pro.EX for Xbox 360"),
        (0x0F0D, 0x001B) => Some("HORI Real Arcade Pro.VX"),
        (0x0F0D, 0x0063) => Some("HORI Real Arcade Pro Hayabusa"),
        (0x0F0D, 0x0067) => Some("HORI Horipad One"),
        (0x0F0D, 0x0078) => Some("HORI Real Arcade Pro V Kai"),
        (0x0F0D, 0x0090) => Some("HORI Horipad Ultimate"),
        (0x0F0D, 0x0092) => Some("HORI Pokken Tournament DX Pro Pad"),
        (0x0F0D, 0x00AA) => Some("HORI Real Arcade Pro V Kai"),
        (0x0F0D, 0x00C1) => Some("HORIPAD for Nintendo Switch"),

        // Mad Catz Controllers, including post-acquisition Saitek aviation
        (0x0738, 0x1302) => Some("Mad Catz F.L.Y. 5 Flight Stick"),
        (0x0738, 0x2215) => Some("Saitek X-55 Rhino Stick"),
        (0x0738, 0x2218) => Some("Saitek Side Panel Control Deck"),
        (0x0738, 0x2237) => Some("Mad Catz V.1 Stick"),
        (0x0738, 0x4506) => Some("Mad Catz Wireless Controller"),
        (0x0738, 0x4516) => Some("Mad Catz Control Pad"),
        (0x0738, 0x4520) => Some("Mad Catz Control Pad Pro"),
        (0x0738, 0x4526) => Some("Mad Catz Control Pad Pro"),
        (0x0738, 0x4530) => Some("Mad Catz Universal MC2 Racing Wheel"),
        (0x0738, 0x4556) => Some("Mad Catz Lynx Wireless Controller"),
        (0x0738, 0x4716) => Some("Mad Catz Wired Xbox 360 Controller"),
        (0x0738, 0x4718) => Some("Mad Catz Street Fighter IV FightStick SE"),
        (0x0738, 0x4726) => Some("Mad Catz Xbox 360 Controller"),
        (0x0738, 0x4728) => Some("Mad Catz Street Fighter IV FightPad"),
        (0x0738, 0x4730) => Some("Mad Catz MC2 Racing Wheel for Xbox 360"),
        (0x0738, 0x4736) => Some("Mad Catz MicroCon for Xbox 360"),
        (0x0738, 0x4738) => Some("Mad Catz Street Fighter IV Wired Controller"),
        (0x0738, 0x4740) => Some("Mad Catz Beat Pad for Xbox 360"),
        (0x0738, 0x4758) => Some("Mad Catz Arcade Game Stick"),
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
        (0x28DE, 0x1102) => Some("Steam Controller (Wired)"),
        (0x28DE, 0x1142) => Some("Steam Controller (Wireless)"),
        (0x28DE, 0x1205) => Some("Steam Deck"),
        (0x28DE, 0x2000) => Some("Valve Lighthouse FPGA RX"),
        (0x28DE, 0x2012) => Some("Valve Virtual Reality Controller"),
        (0x28DE, 0x2101) => Some("Valve Watchman Dongle"),
        (0x28DE, 0x2500) => Some("Valve Lighthouse Base Station"),

        // 0x24C6 is assigned to ThrustMaster in usb.ids. Many Hori and
        // PowerA licensed Xbox controllers ship under the same VID.
        (0x24C6, 0x551A) => Some("PowerA Xbox One Enhanced Wired Controller"),
        (0x24C6, 0x5500) => Some("Hori Horipad EX2 Turbo"),
        (0x24C6, 0x5501) => Some("Hori Real Arcade Pro VX-SA (Xbox 360)"),
        (0x24C6, 0x5B00) => Some("Mad Catz Ferrari 458 Italia Racing Wheel"),

        // GameSir on the newer VID 0x3537
        (0x3537, 0x1001) => Some("GameSir T4 Pro"),
        (0x3537, 0x1003) => Some("GameSir G7"),
        (0x3537, 0x1004) => Some("GameSir T4 Kaleid"),
        (0x3537, 0x1005) => Some("GameSir Cyclone"),
        (0x3537, 0x1007) => Some("GameSir Nova"),

        // Flydigi Controllers
        (0x32C2, 0x0101) => Some("Flydigi Apex 2"),
        (0x32C2, 0x0102) => Some("Flydigi Apex 3"),
        (0x32C2, 0x1104) => Some("Flydigi Vader 2"),
        (0x32C2, 0x1107) => Some("Flydigi Vader 3"),
        (0x32C2, 0x110E) => Some("Flydigi Vader 4 Pro"),
        (0x32C2, 0x1201) => Some("Flydigi Direwolf 2"),

        // BigBigWon
        (0x33DF, 0x0001) => Some("BigBigWon Rainbow 2 Pro"),
        (0x33DF, 0x0010) => Some("BigBigWon Blitz 2"),

        // Google Stadia
        (0x18D1, 0x9400) => Some("Stadia Controller"),

        // Meta/Oculus
        (0x2833, 0x0001) => Some("Oculus Rift Developer Kit 1"),
        (0x2833, 0x0021) => Some("Oculus Rift DK2"),
        (0x2833, 0x0031) => Some("Oculus Rift CV1"),
        (0x2833, 0x0101) => Some("Oculus Latency Tester"),
        (0x2833, 0x0137) => Some("Meta Quest Headset"),
        (0x2833, 0x0201) => Some("Oculus Rift DK2 Camera"),
        (0x2833, 0x0211) => Some("Oculus Rift CV1 Sensor"),
        (0x2833, 0x0330) => Some("Oculus Rift CV1 Audio"),

        // HTC Vive
        (0x0BB4, 0x0306) => Some("HTC Vive Hub (Bluetooth)"),
        (0x0BB4, 0x2134) => Some("HTC Vive Hub (SMSC USB2137B)"),
        (0x0BB4, 0x2744) => Some("HTC Vive Hub (HTC CB USB2)"),
        (0x0BB4, 0x2C87) => Some("HTC Vive"),

        // Nacon Controllers
        (0x3285, 0x0003) => Some("Nacon GC-400ES"),
        (0x3285, 0x0010) => Some("Nacon Revolution Pro Controller"),
        (0x3285, 0x0607) => Some("Nacon GC-100"),

        // ThrustMaster
        (0x044F, 0x0400) => Some("ThrustMaster HOTAS Cougar"),
        (0x044F, 0x0402) => Some("ThrustMaster HOTAS Warthog Joystick"),
        (0x044F, 0x0404) => Some("ThrustMaster HOTAS Warthog Throttle"),
        (0x044F, 0x044F) => Some("ThrustMaster GP XID Gamepad"),
        (0x044F, 0x0F00) => Some("ThrustMaster Xbox Steering Wheel"),
        (0x044F, 0x0F07) => Some("ThrustMaster Xbox Controller"),
        (0x044F, 0x0F10) => Some("ThrustMaster Modena GT Wheel"),
        (0x044F, 0xA003) => Some("ThrustMaster Rage 3D Game Pad"),
        (0x044F, 0xA0A0) => Some("ThrustMaster Top Gun Joystick"),
        (0x044F, 0xA201) => Some("ThrustMaster PK-GP201 PlayStick"),
        (0x044F, 0xB108) => Some("ThrustMaster T-Flight HOTAS X"),
        (0x044F, 0xB10A) => Some("ThrustMaster T.16000M"),
        (0x044F, 0xB203) => Some("ThrustMaster 360 Modena Pro Wheel"),
        (0x044F, 0xB300) => Some("ThrustMaster Firestorm Dual Power"),
        (0x044F, 0xB307) => Some("ThrustMaster Vibrating Upad"),
        (0x044F, 0xB315) => Some("ThrustMaster Firestorm Dual Analog 3"),
        (0x044F, 0xB320) => Some("ThrustMaster Dual Trigger Gamepad PC/PS2"),
        (0x044F, 0xB326) => Some("ThrustMaster Gamepad GP XID"),
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
        (0x046D, 0xC541) => Some("Logitech G903 HERO"),
        (0x046D, 0xC547) => Some("Logitech G PRO Wireless"),
        (0x046D, 0xC548) => Some("Logitech Logi Bolt Receiver"),
        (0x046D, 0xC534) => Some("Logitech Unifying Receiver"),
        (0x046D, 0xC52B) => Some("Logitech Unifying Receiver"),
        (0x046D, 0xC33A) => Some("Logitech G413"),
        (0x046D, 0xC33C) => Some("Logitech G513 Carbon"),
        (0x046D, 0xC33F) => Some("Logitech G815"),
        (0x046D, 0xC343) => Some("Logitech G915"),
        (0x046D, 0xC344) => Some("Logitech G915 TKL"),
        (0x046D, 0xC335) => Some("Logitech G610 Orion"),
        (0x046D, 0xC336) => Some("Logitech G810 Orion Spectrum"),
        (0x046D, 0xC339) => Some("Logitech G Pro Mechanical Keyboard"),
        (0x046D, 0xC077) => Some("Logitech M105 Mouse"),

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
        (0x1532, 0x0088) => Some("Razer Basilisk Ultimate"),
        (0x1532, 0x008A) => Some("Razer Viper 8KHz"),
        (0x1532, 0x008B) => Some("Razer Basilisk X HyperSpeed"),
        (0x1532, 0x0094) => Some("Razer Viper Mini"),
        (0x1532, 0x009D) => Some("Razer Naga Pro"),
        (0x1532, 0x00A5) => Some("Razer Viper V2 Pro"),
        (0x1532, 0x00B8) => Some("Razer DeathAdder V2 X HyperSpeed"),
        (0x1532, 0x00CE) => Some("Razer Cobra Pro"),
        (0x1532, 0x00B6) => Some("Razer Basilisk V3"),
        (0x1532, 0x007A) => Some("Razer DeathAdder V2"),
        (0x1532, 0x0216) => Some("Razer BlackWidow Ultimate 2013"),
        (0x1532, 0x0221) => Some("Razer BlackWidow Chroma"),
        (0x1532, 0x0226) => Some("Razer Ornata"),
        (0x1532, 0x0227) => Some("Razer BlackWidow X Chroma"),
        (0x1532, 0x022D) => Some("Razer Huntsman"),
        (0x1532, 0x022F) => Some("Razer Huntsman Tournament Edition"),
        (0x1532, 0x0233) => Some("Razer Huntsman Elite"),
        (0x1532, 0x0241) => Some("Razer BlackWidow V3 Pro"),
        (0x1532, 0x0243) => Some("Razer Huntsman Mini"),
        (0x1532, 0x0253) => Some("Razer BlackWidow V4 Pro"),
        (0x1532, 0x0257) => Some("Razer Huntsman V3 Pro"),
        (0x1532, 0x028D) => Some("Razer BlackWidow V4 75%"),
        (0x1532, 0x0510) => Some("Razer Kraken 7.1"),
        (0x1532, 0x0520) => Some("Razer Kraken 7.1 Chroma"),
        (0x1532, 0x0527) => Some("Razer Kraken Ultimate"),
        (0x1532, 0x0533) => Some("Razer BlackShark V2 Pro"),

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

        // GameSir on the older OEM VID 0x0E8F, superseded by 0x3537
        (0x0E8F, 0x3013) => Some("GameSir G3w"),

        // Jess Technology and Qanba Arcade Sticks. usb.ids assigns VID
        // 0x0F30 to Jess Technology and Qanba ships under the same VID.
        (0x0F30, 0x001C) => Some("Jess PS3 Guitar Controller Dongle"),
        (0x0F30, 0x010B) => Some("Philips Recoil"),
        (0x0F30, 0x0110) => Some("Jess Dual Analog Rumble Pad"),
        (0x0F30, 0x0111) => Some("Jess Colour Rumble Pad / Qanba Q4RAF"),
        (0x0F30, 0x0202) => Some("Joytech Advanced Controller"),
        (0x0F30, 0x0208) => Some("Jess Xbox & PC Gamepad"),
        (0x0F30, 0x00F8) => Some("Qanba Drone"),
        (0x0F30, 0x1100) => Some("Qanba Obsidian"),
        (0x0F30, 0x8888) => Some("BigBen XBMiniPad Controller"),

        // Saitek / extended flight sim peripherals
        (0x06A3, 0x040B) => Some("Saitek P880 Pad"),
        (0x06A3, 0x040C) => Some("Saitek P2900 Wireless Pad"),
        (0x06A3, 0x0460) => Some("Saitek ST290 Pro Flight Stick"),
        (0x06A3, 0x0464) => Some("Saitek Cyborg Evo"),
        (0x06A3, 0x0471) => Some("Saitek Cyborg Graphite Stick"),
        (0x06A3, 0x053C) => Some("Saitek X45 Flight Controller"),
        (0x06A3, 0x053F) => Some("Saitek X36F Flightstick"),
        (0x06A3, 0x052D) => Some("Saitek P750 Gamepad"),
        (0x06A3, 0x0763) => Some("Saitek Pro Flight Rudder Pedals"),
        (0x06A3, 0x0805) => Some("Saitek R440 Force Wheel"),

        // CH Products flight simulators
        (0x068E, 0x00F1) => Some("CH Pro Throttle"),
        (0x068E, 0x00F2) => Some("CH Flight Sim Pedals"),
        (0x068E, 0x00F3) => Some("CH Fighterstick"),
        (0x068E, 0x00F4) => Some("CH Combatstick"),
        (0x068E, 0x00FA) => Some("CH Throttle Quadrant"),
        (0x068E, 0x00FF) => Some("CH Flight Sim Yoke"),
        (0x068E, 0x0500) => Some("CH GameStick 3D"),
        (0x068E, 0x0501) => Some("CH Pro Pedals"),
        (0x068E, 0x0504) => Some("CH F-16 Combat Stick"),

        // Guillemot / Hercules
        (0x06F8, 0xA300) => Some("Hercules Dual Analog Leader GamePad"),
        (0x06F8, 0xB000) => Some("Hercules DJ Console"),
        (0x06F8, 0xB105) => Some("Hercules DJ Control MP3 e2"),
        (0x06F8, 0xB121) => Some("Hercules P32 DJ"),

        // GreenAsia / generic gamepad adapters
        (0x0E8F, 0x0003) => Some("GreenAsia MaxFire Blaze2"),
        (0x0E8F, 0x0012) => Some("GreenAsia Joystick/Gamepad"),
        (0x0E8F, 0x0201) => Some("SmartJoy Frag Xpad/PS2 Adapter"),
        (0x0E8F, 0x3008) => Some("GreenAsia Xbox Controller"),
        (0x0E8F, 0x300A) => Some("GreenAsia Steering Wheel"),

        // Harmonix Music rhythm game peripherals
        (0x1BAD, 0x0002) => Some("Harmonix Rock Band Guitar for Xbox 360"),
        (0x1BAD, 0x0003) => Some("Harmonix Rock Band Drum Kit for Xbox 360"),
        (0x1BAD, 0x0130) => Some("Ion Drum Rocker for Xbox 360"),
        (0x1BAD, 0xF018) => Some("Harmonix Street Fighter IV SE FightStick"),
        (0x1BAD, 0xF023) => Some("MLG Pro Circuit Controller for Xbox 360"),
        (0x1BAD, 0xF025) => Some("Call of Duty Controller for Xbox 360"),
        (0x1BAD, 0xF028) => Some("Street Fighter IV FightPad for Xbox 360"),
        (0x1BAD, 0xF03A) => Some("SF X Tekken FightStick Pro for Xbox 360"),
        (0x1BAD, 0xF03D) => Some("SF IV Arcade Stick TE for Xbox 360"),
        (0x1BAD, 0xF042) => Some("Arcade FightStick TE S+ for Xbox 360"),
        (0x1BAD, 0xF080) => Some("FightStick TE2 for Xbox 360"),
        (0x1BAD, 0xF501) => Some("Horipad EX2 Turbo for Xbox 360"),
        (0x1BAD, 0xF506) => Some("Real Arcade Pro.EX Premium VLX"),

        // RedOctane rhythm game hardware
        (0x1430, 0x0150) => Some("RedOctane Skylanders Wireless Receiver"),
        (0x1430, 0x4734) => Some("Guitar Hero 4 Hub"),
        (0x1430, 0x4748) => Some("Guitar Hero X-plorer"),
        (0x1430, 0x474B) => Some("Guitar Hero MIDI Interface"),
        (0x1430, 0x8888) => Some("RedOctane TX6500+ Dance Pad"),

        // Microsoft Xbox 360 Kinect and accessories
        (0x045E, 0x02AD) => Some("Xbox Kinect NUI Audio"),
        (0x045E, 0x02AE) => Some("Xbox Kinect NUI Camera"),
        (0x045E, 0x02B0) => Some("Xbox Kinect NUI Motor"),
        (0x045E, 0x02B6) => Some("Xbox 360 Bluetooth Wireless Headset"),
        (0x045E, 0x02E6) => Some("Xbox Wireless Adapter for Windows"),
        (0x045E, 0x02FE) => Some("Xbox Wireless Adapter for Windows (rev2)"),
        (0x045E, 0x02F3) => Some("Xbox One Chatpad"),

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

        // Keychron mechanical keyboards
        (0x3434, 0x0111) => Some("Keychron K1"),
        (0x3434, 0x0120) => Some("Keychron K2"),
        (0x3434, 0x0130) => Some("Keychron K3"),
        (0x3434, 0x0311) => Some("Keychron Q1"),
        (0x3434, 0x0320) => Some("Keychron Q2"),
        (0x3434, 0x0330) => Some("Keychron Q3"),
        (0x3434, 0x0361) => Some("Keychron Q6"),
        (0x3434, 0x03A3) => Some("Keychron Q10"),
        (0x3434, 0x0A30) => Some("Keychron V3"),
        (0x3434, 0x0280) => Some("Keychron K8"),

        // Akko keyboards. Some models also appear under SINO WEALTH.
        (0x320F, 0x5066) => Some("Akko 3098 DS"),
        (0x320F, 0x5087) => Some("Akko 5087"),
        (0x320F, 0x5108) => Some("Akko 5108"),
        (0x320F, 0x5075) => Some("Akko 5075B"),

        // Wooting analog-switch keyboards
        (0x31E3, 0x1100) => Some("Wooting One"),
        (0x31E3, 0x1200) => Some("Wooting Two"),
        (0x31E3, 0x1210) => Some("Wooting Two HE"),
        (0x31E3, 0x1300) => Some("Wooting 60HE"),
        (0x31E3, 0x1310) => Some("Wooting 60HE+"),

        // NuPhy keyboards
        (0x3151, 0x4004) => Some("NuPhy Air75"),
        (0x3151, 0x4006) => Some("NuPhy Halo75"),
        (0x3151, 0x4008) => Some("NuPhy Field75"),

        // Elgato streaming hardware
        (0x0FD9, 0x0060) => Some("Elgato Stream Deck"),
        (0x0FD9, 0x0063) => Some("Elgato Stream Deck Mini"),
        (0x0FD9, 0x006C) => Some("Elgato Stream Deck XL"),
        (0x0FD9, 0x006D) => Some("Elgato Stream Deck Mobile"),
        (0x0FD9, 0x0080) => Some("Elgato Stream Deck MK.2"),
        (0x0FD9, 0x0084) => Some("Elgato Stream Deck +"),
        (0x0FD9, 0x00AB) => Some("Elgato Stream Deck Neo"),

        // XP-PEN drawing tablets
        (0x28BD, 0x0075) => Some("XP-PEN Deco 01"),
        (0x28BD, 0x0904) => Some("XP-PEN Artist 12"),
        (0x28BD, 0x0905) => Some("XP-PEN Artist 15.6"),
        (0x28BD, 0x0906) => Some("XP-PEN Artist 22"),

        // Huion drawing tablets
        (0x256C, 0x006D) => Some("Huion H610 Pro V2"),
        (0x256C, 0x006E) => Some("Huion Kamvas"),

        // Apple input devices
        (0x05AC, 0x024F) => Some("Apple Keyboard"),
        (0x05AC, 0x0250) => Some("Apple Aluminum Keyboard"),
        (0x05AC, 0x0259) => Some("Apple Magic Keyboard"),
        (0x05AC, 0x030D) => Some("Apple Magic Mouse 2"),
        (0x05AC, 0x030E) => Some("Apple Magic Trackpad 2"),
        (0x05AC, 0x0265) => Some("Apple Magic Mouse"),

        // Microsoft productivity peripherals
        (0x045E, 0x07F8) => Some("Microsoft Sculpt Ergonomic Keyboard"),
        (0x045E, 0x00DB) => Some("Microsoft Natural Ergonomic Keyboard 4000"),
        (0x045E, 0x0773) => Some("Microsoft Wedge Keyboard"),
        (0x045E, 0x07A5) => Some("Microsoft Wireless Desktop Receiver"),
        (0x045E, 0x0745) => Some("Microsoft Nano Transceiver v1.0"),
        (0x045E, 0x0040) => Some("Microsoft Wheel Mouse Optical"),

        // Glorious / SINO WEALTH mice
        (0x258A, 0x002A) => Some("Glorious Model O"),
        (0x258A, 0x0039) => Some("Glorious Model D"),
        (0x258A, 0x004B) => Some("Glorious Model O Wireless"),

        _ => None,
    }
}

/// Maps HID Usage Page and Usage to a human-readable device type.
///
/// Values follow the USB HID Usage Tables v1.4 spec. Usage Page 0x01 is
/// Generic Desktop, 0x08 is LEDs, 0x0B is Telephony, 0x0C is Consumer,
/// 0x0D is Digitizers, and 0x0F is Physical Input Device.
#[inline]
pub fn get_hid_device_type(usage_page: u16, usage: u16) -> &'static str {
    match (usage_page, usage) {
        // Generic Desktop page.
        (0x0001, 0x0001) => "Pointer",
        (0x0001, 0x0002) => "Mouse",
        (0x0001, 0x0004) => "Joystick",
        (0x0001, 0x0005) => "Gamepad",
        (0x0001, 0x0006) => "Keyboard",
        (0x0001, 0x0007) => "Keypad",
        (0x0001, 0x0008) => "Multi-axis Controller",
        (0x0001, 0x0009) => "Tablet PC System Controls",
        (0x0001, 0x000D) => "Portable Device Control",
        (0x0001, 0x000E) => "Interactive Control",
        (0x0001, 0x0080) => "System Control",
        // Consumer page for media keys and remote controls.
        (0x000C, 0x0001) => "Consumer Control",
        (0x000C, 0x0080) => "Selection Control",
        // Digitizers page covers pens and touch surfaces.
        (0x000D, 0x0001) => "Digitizer",
        (0x000D, 0x0002) => "Pen",
        (0x000D, 0x0004) => "Touch Screen",
        (0x000D, 0x0005) => "Touch Pad",
        // Physical Input Device page handles force feedback.
        (0x000F, 0x0001) => "Force Feedback Device",
        // LEDs page.
        (0x0008, 0x0001) => "LED Indicator",
        // Telephony page.
        (0x000B, 0x0001) => "Phone",
        (0x000B, 0x0005) => "Headset",
        _ => "HID Device",
    }
}
