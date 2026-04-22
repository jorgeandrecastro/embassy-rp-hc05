[![crates.io](https://img.shields.io/crates/v/embassy-rp-hc05.svg)](https://crates.io/crates/embassy-rp-hc05)
[![docs.rs](https://docs.rs/embassy-rp-hc05/badge.svg)](https://docs.rs/embassy-rp-hc05)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

# embassy-rp-hc05

Driver async `no_std` minimaliste pour le module Bluetooth série **HC-05**,
testé sur microcontrôleur **RP2040** à tester sur **RP235x**, basé sur le framework [Embassy](https://embassy.dev).

---

## Description

Le **HC-05** est un module Bluetooth Classic (SPP: Serial Port Profile) qui expose
une interface UART simple. Il permet de créer une liaison série sans fil entre votre
microcontrôleur et un smartphone, PC ou autre appareil Bluetooth.

Ce driver encapsule l'UART asynchrone d'Embassy et expose une API simple pour envoyer des données, détecter l'état de la connexion et recevoir des commandes via la fonction read_line. Fidèle à la philosophie de Rust, ce projet impose #![forbid(unsafe_code)] pour garantir une sécurité mémoire absolue.

---

## Câblage

Connexion minimale :

```
RP2040 / RP235x          HC-05
────────────────          ─────
TX  (ex: GP0)  ─────►  RX
RX  (ex: GP1)  ◄─────  TX
3.3V           ─────►  VCC  (certains modules acceptent 5V)
GND            ─────►  GND
GP2 (optionnel)◄─────  STATE  (HIGH = connecté)
```

> **Note :** La pin `STATE` est optionnelle mais recommandée pour détecter
> si un appareil est effectivement connecté au HC-05.

---

## Installation

Ajoutez la dépendance dans votre `Cargo.toml` :

**Pour le RP2040 feature par défaut**
```toml
[dependencies.embassy-rp-hc05]
version = "0.1.0"
```

**Pour le RP235x**
```toml
[dependencies]
embassy-rp-hc05 = { version = "0.1.0", default-features = false, features = ["rp235x"] }
```

> **Compatibilité :** Cette crate supporte `embassy-rp` de la version `0.4.0` à `0.10.x+`.

---

## Features

| Feature    | Description                                 | Par défaut |
|------------|---------------------------------------------|------------|
| `rp2040`   | Active le support RP2040                    | ✅ oui     |
| `rp235x`   | Active le support RP235x                    | ❌ non     |

---

## API

### `BluetoothHandler::new(uart, state_pin) -> Self`

Crée le driver en prenant possession de l'UART Embassy et d'une pin `STATE` optionnelle.

```rust
let mut bt = BluetoothHandler::new(uart, Some(state_input));
// ou sans pin STATE :
let mut bt = BluetoothHandler::new(uart, None);
```

---

### `fn is_connected(&self) -> bool`

Vérifie l'état physique de la connexion Bluetooth via la pin `STATE` du HC-05.

- Si la pin `STATE` est fournie : retourne `true` quand elle est `HIGH` (connexion active).
- Si aucune pin n'est fournie : retourne toujours `true`.

```rust
if bt.is_connected() {
    // un appareil Bluetooth est connecté
}
```

---

### `async fn send(&mut self, message: &str) -> Result<(), BluetoothError>`

Envoie une chaîne de caractères brute via UART (sans retour chariot).

```rust
bt.send("OK").await?;
bt.send("Temperature:").await?;
```

---

### `async fn send_line(&mut self, message: &str) -> Result<(), BluetoothError>`

Envoie un message **suivi de `\r\n`**. Pratique pour les terminaux série ou
les parsers ligne-par-ligne côté récepteur.

```rust
bt.send_line("Bonjour depuis le microcontroleur!").await?;
// envoie : "Bonjour depuis le microcontroleur!\r\n"
```

---

### `async fn send_i16(&mut self, val: i16) -> Result<(), BluetoothError>`

Envoie un entier signé 16 bits converti en texte ASCII. Utilise [`itoa`](https://crates.io/crates/itoa) sans allocation.

```rust
bt.send_i16(-1234).await?;
// envoie : "-1234"
```

---

### `async fn send_i16_line(&mut self, val: i16) -> Result<(), BluetoothError>`

Envoie un entier `i16` suivi de `\r\n`.

```rust
bt.send_i16_line(-42).await?;
// envoie : "-42\r\n"
```

---

### `async fn send_u16(&mut self, val: u16) -> Result<(), BluetoothError>`

Envoie un entier non signé 16 bits converti en texte ASCII.
Pratique pour transmettre des valeurs ADC brutes (0–4095 sur RP2040, 0–16383 sur RP235x).

```rust
bt.send_u16(3012).await?;
// envoie : "3012"
```

---

### `async fn send_u16_line(&mut self, val: u16) -> Result<(), BluetoothError>`

Envoie un entier `u16` suivi de `\r\n`.

```rust
bt.send_u16_line(4095).await?;
// envoie : "4095\r\n"
```

---

### `async fn send_u32(&mut self, val: u32) -> Result<(), BluetoothError>`

Envoie un entier non signé 32 bits converti en texte ASCII.

```rust
bt.send_u32(123456).await?;
```

---

### `async fn read(&mut self, buffer: &mut [u8]) -> Result<(), BluetoothError>`

Lit des octets depuis l'UART dans le buffer fourni.
Bloque jusqu'à ce que le buffer soit entièrement rempli.

```rust
let mut buf = [0u8; 32];
bt.read(&mut buf).await?;
// buf contient maintenant les données reçues
```

---

### `enum BluetoothError`

```rust
pub enum BluetoothError {
    Uart(UartError),  // Erreur UART sous-jacente Embassy
}
```

### `async fn read_line(&mut self, buf: &mut [u8]) -> Result<usize, BluetoothError>`
Lit les données reçues via Bluetooth jusqu'à rencontrer un caractère de fin de ligne (\n) ou jusqu'à ce que le buffer soit plein. Cette méthode est asynchrone : elle suspend la tâche en cours sans bloquer le processeur tant que des données ne sont pas disponibles.

Fonctionnement :

Remplit le buffer fourni buf octet par octet.

S'arrête dès qu'un \n (Line Feed) est détecté.

Retourne le nombre d'octets réellement lus (usize).

````rust
let mut rx_buf = [0u8; 32];

// Attend que l'utilisateur envoie une commande (ex: "ON\n")
if let Ok(n) = bt.read_line(&mut rx_buf).await {
    // n contient la longueur du message reçu
    let message = core::str::from_utf8(&rx_buf[..n]).unwrap_or("");
    
    if message.contains("ON") {
        // Faire quelque chose...
    }
}
````
Note technique : Si le buffer est plein avant d'avoir trouvé un \n, la fonction retourne la taille totale du buffer. Il est conseillé de vider ou d'analyser le buffer pour ne pas perdre le reste du message.

---

## Utilisation complète

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::uart::{Config as UartConfig, InterruptHandler, Uart};
use embassy_rp::peripherals::UART0;
use embassy_time::{Duration, Timer};
use embassy_rp_hc05::BluetoothHandler;
use {panic_halt as _, embassy_rp as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => InterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Configuration UART — baudrate 9600 par défaut pour HC-05
    let mut uart_config = UartConfig::default();
    uart_config.baudrate = 9600;

    let uart = Uart::new(
        p.UART0,
        p.PIN_0, // TX
        p.PIN_1, // RX
        Irqs,
        p.DMA_CH0,
        p.DMA_CH1,
        uart_config,
    );

    // Pin STATE optionnelle (HIGH = connexion active)
    let state = Input::new(p.PIN_2, Pull::Down);

    let mut bt = BluetoothHandler::new(uart, Some(state));

    loop {
        if bt.is_connected() {
            bt.send_line("Microcontroleur connecte!").await.ok();
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}
```

---

## Exemple avec ADC envoi de valeur lumineuse via Bluetooth

Combiné avec [`embassy-rp-gl5528`](https://crates.io/crates/embassy-rp-gl5528) :

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig, InterruptHandler as AdcIrq};
use embassy_rp::bind_interrupts;
use embassy_rp::uart::{Config as UartConfig, InterruptHandler as UartIrq, Uart};
use embassy_rp::peripherals::{ADC, UART0};
use embassy_time::{Duration, Timer};
use embassy_rp_gl5528::Gl5528;
use embassy_rp_hc05::BluetoothHandler;
use {panic_halt as _, embassy_rp as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcIrq;
    UART0_IRQ    => UartIrq<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // ADC + capteur lumière
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let channel = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);
    let mut sensor = Gl5528::new(adc, channel);

    // UART + Bluetooth
    let mut uart_config = UartConfig::default();
    uart_config.baudrate = 9600;
    let uart = Uart::new(p.UART0, p.PIN_0, p.PIN_1, Irqs, p.DMA_CH0, p.DMA_CH1, uart_config);
    let mut bt = BluetoothHandler::new(uart, None);

    loop {
        let raw = sensor.read_raw().await;

        bt.send("Lux:").await.ok();
        bt.send_u16_line(raw).await.ok();

        Timer::after(Duration::from_millis(500)).await;
    }
}
```
# Exemple Sigmoide , Enoie de messages asynchrone et affichage Oled sans lock
Combiné avec [`embassy-rp-gl5528`](https://crates.io/crates/embassy-rp-gl5528)
et [`sigmoid-q15`](https://crates.io/crates/sigmoid-q15) et [`embassy-ssd1306`](https://crates.io/crates/embassy-ssd1306)


````rust
#![no_std]
#![no_main]

use cortex_m_rt as _;
use embassy_executor::Spawner;
use embassy_rp::i2c::{Config as I2cConfig, I2c, Async as I2cAsync};
use embassy_time::Timer; 
use {panic_halt as _, embassy_rp as _};

use embassy_ssd1306::Ssd1306;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{I2C0, UART0}; 
use rp2040_linker as _; 

use embassy_rp::gpio::{Output, Level, Pull, Input};
use embassy_rp::adc::{Adc, Config as AdcConfig, Channel, InterruptHandler as AdcInterruptHandler};
use embassy_rp::i2c::InterruptHandler as I2cInterruptHandler;
use embassy_rp::uart::{Uart, Config as UartConfig, InterruptHandler as UartInterruptHandler};

use sigmoid_q15::sigmoid_q15;
use embassy_rp_gl5528::Gl5528; 
use static_cell::StaticCell;

// On utilise la crate
use embassy_rp_hc05::BluetoothHandler;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2cInterruptHandler<I2C0>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
    UART0_IRQ => UartInterruptHandler<UART0>;
});

// IMPORTANT : Le Mutex permet aux deux tâches d'accéder au BT sans se bloquer
static BT_HANDLER: StaticCell<Mutex<NoopRawMutex, BluetoothHandler<'static>>> = StaticCell::new();

// TÂCHE 1 : L'AFFICHAGE ET L'ENVOI (Fluide)
#[embassy_executor::task]
async fn ui_task(
    mut oled: Ssd1306<I2cDevice<'static, NoopRawMutex, I2c<'static, I2C0, I2cAsync>>>,
    mut light_sensor: Gl5528<'static>,
    bt_mutex: &'static Mutex<NoopRawMutex, BluetoothHandler<'static>>,
) {
    let _ = oled.init().await;
    let test_points: [i16; 5] = [i16::MIN, -16384, 0, 16384, i16::MAX];
    let mut idx = 0;

    loop {
        let x_in = test_points[idx];
        let y_out = sigmoid_q15(x_in);
        let lux_raw = light_sensor.read_raw().await;

        //  TENTATIVE D'ENVOI SANS BLOQUER 
        // try_lock() permet de vérifier si la clé est disponible. 
        // Si la rx_task attend un message, try_lock() renverra None,
        // et on passera directement à l'affichage OLED !
        if let Ok(mut bt) = bt_mutex.try_lock() {
            if bt.is_connected() {
                let _ = bt.send("Sig: ").await;
                let _ = bt.send_i16_line(x_in).await;
                let _ = bt.send_i16_line(y_out).await;
            }
        }

        //  MISE À JOUR OLED (S'exécute quoi qu'il arrive) 
        oled.clear();
        oled.draw_str(10, 0, b"SYSTEME ACTIF");
        oled.draw_str(0, 2, b"X_in:");
        oled.draw_i16(40, 2, x_in);
        oled.draw_str(0, 4, b"Sig:");
        oled.draw_i16(40, 4, y_out);
        oled.draw_str(0, 6, b"Lux:");
        oled.draw_i16(40, 6, lux_raw as i16);
        let _ = oled.flush().await;

        idx = (idx + 1) % test_points.len();
        Timer::after_millis(500).await;
    }
}

//  TÂCHE 2 : LA RÉCEPTION (Elle peut freezer, elle est toute seule) 
#[embassy_executor::task]
async fn rx_task(
    bt_mutex: &'static Mutex<NoopRawMutex, BluetoothHandler<'static>>,
) {
    let mut rx_buf = [0u8; 32];

    loop {
        let mut n = 0;
        {
            let mut bt = bt_mutex.lock().await;
            if bt.is_connected() {
                // Cette ligne attend (freeze cette tâche), mais pas l'OLED !
                if let Ok(count) = bt.read_line(&mut rx_buf).await {
                    n = count;
                }
            }
        }

        if n > 0 {
            let mut bt = bt_mutex.lock().await;
            let _ = bt.send("Pico a bien recu: ").await;
            if let Ok(msg) = core::str::from_utf8(&rx_buf[..n]) {
                let _ = bt.send(msg).await;
            }
        }
        Timer::after_millis(10).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    
    // I2C & OLED
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 400_000; 
    let i2c_bus = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, i2c_config);
    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, I2c<'static, I2C0, I2cAsync>>> = StaticCell::new();
    let i2c_mutex = I2C_BUS.init(Mutex::new(i2c_bus));
    let oled = Ssd1306::new(I2cDevice::new(i2c_mutex), 0x3C);

    // ADC & CAPTEUR
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let light_sensor = Gl5528::new(adc, Channel::new_pin(p.PIN_26, Pull::None));

    // UART & BLUETOOTH
    let mut uart_cfg = UartConfig::default();
    uart_cfg.baudrate = 9600;
    let uart = Uart::new(p.UART0, p.PIN_0, p.PIN_1, Irqs, p.DMA_CH0, p.DMA_CH1, uart_cfg);
    
    let bt = BluetoothHandler::new(uart, Some(Input::new(p.PIN_2, Pull::None)));
    let bt_shared = BT_HANDLER.init(Mutex::new(bt));

    // ON LANCE LES DEUX EN MÊME TEMPS
    spawner.spawn(ui_task(oled, light_sensor, bt_shared)).unwrap();
    spawner.spawn(rx_task(bt_shared)).unwrap();

    let mut led = Output::new(p.PIN_25, Level::Low);
    loop {
        led.toggle();
        Timer::after_millis(200).await;
    }
}
````

---

## Compatibilité

| Dépendance   | Version       |
|--------------|---------------|
| `embassy-rp` | 0.4 à 0.10+   |
| `itoa`       | >=1.0, <2     |
| Rust edition | 2024          |
| `no_std`     | ✓             |

---

## Historique et Compatibilité

Il est recommandé d'utiliser la version **0.1.0 ou supérieure**.

Cette crate est compatible avec une large plage de versions d'`embassy-rp` (v0.4.0 à v0.10.0+).
Si vous rencontrez un problème de compilation ou un comportement inattendu, n'hésitez pas à
ouvrir une **Issue GitHub** votre retour est précieux !

Consultez le fichier `CHANGELOG.md` pour le détail des changements.

---

## Licence

Ce projet est distribué sous licence **GPL-2.0-or-later**.
Voir le fichier [LICENSE](LICENSE) pour les détails complets.

---

## 🦅 À propos

Développé et testé par Jorge Andre Castro