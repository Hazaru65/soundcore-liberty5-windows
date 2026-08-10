# Liberty 5 protocol notes

## Durum (2026-08-11, gercek cihazda dogrulandi)

Kontrol kanali **Bluetooth Classic RFCOMM**, BLE degil. BLE yalnizca jenerik servisler sunar (0x1800/0x1801); Battery Service 0x180F yok.

## Dogrulanmis protokol

- **Kontrol servisi:** `0cf12d31-fac3-4553-bd80-d6832e7b3957` (RFCOMM)
- **Cerceve yapisi** (Soundcore v1, kablodaki bayt siralariyla):
  - Host -> cihaz: `08 ee | 00 00 | 00 | komut u16-LE | uzunluk u16-LE | payload | saplama`
  - Cihaz -> host: `09 ff | 00 00 | 01 | komut u16-LE | uzunluk u16-LE | payload | saplama`
  - uzunluk = tum cerceve boyutu; payload = uzunluk - 10 bayt; son bayt saplama = onceki tum baytlarin toplami
- **0x0101 cihaz bilgisi** (payload bos): yanit 164 bayt; seri [16..32], firmware1 [6..11], firmware2 [11..16]
  - Gercek ornek: seri `395790BFD95CE5DC`, firmware `04.90`
- **0x8106 ANC ayari**: payload `[mod, 0x10, 0x00, 0x01, 0x00, 0x01]`; mod 0x00=ANC, 0x01=Transparency, 0x02=Off
  - Uc mod da gercek cihazda yazildi; her biri ayni komut kodlu bos payload ACK'i ile dondu
  - Telefon uygulamasi formati: `[mod, 0x51, 0x00, 0x01, 0x00, 0x00, 0x00]` (HCI yakalamasindan; her iki format da cihazda calisiyor)
- **0x8510 Game Mode**: payload `[0x01]`=acik, `[0x00]`=kapali. HCI yakalamasindan alindi; gercek cihazda isitsel olarak dogrulandi (acikken ses boguklasiyor/gecikme optimize, kapaliyken netlesiyor). Her yazimdan sonra `0x017f [00]` bildirimi geliyor (semantik bilinmiyor)
- **0x0301 pil**: bos payload istek -> yanit `[sol][sag][kutu]`, yuzde = (deger+1)*10. Dogrulandi: `09 06 06` -> %100/%70/%70 (uygulama ekraniyla birebir; AeroFit formuluyle ayni). `0x0101` yaniti payload[2..3] de sol/sag pil degerlerini tasir
- **0x0106 bildirimi**: mevcut ses modu, payload[0] ayni 0x00/0x01/0x02 eslemesi; baglanti aninda zaman zaman geliyor
- **0x0103**: baglanti aninda gelen kucuk bilgi cercevesi (semantik bilinmiyor)
- **0x020b**: `[flag] + 6 bayt MAC` bildirimi; ornek `00 2a10f18c5bfc` / `01 2a10f18c5bfc` (bagli cihaz / multipoint adayi; semantik dogrulanmadi)

## Dogrulanamayanlar (kilitli kalir)

- **Game Mode**: ~~AeroFit 2 komutu `0x8701` reddedildi~~ — **COZULDU**: gercek komut `0x8510` (yukariya bak). `0x8701` Liberty 5 tarafindan taninmiyor.
- **EQ**: `0x8703` (114 bayt band verisi, preset id 01/02/03/00) ve `0x8110` (preset secim adayi) HCI yakalamasindan alindi, cihazda ACK aliyor; **isitsel dogrulama yapilmadi** — kullanici EQ'yu sonraya birakti. Aday komutlar: `0x8703` = band ayari, `0x8110 [01 01 XX]` = preset/secim, `0x8510` haric tutuldu. Preset id -> isim eslemesi (uygulamadaki preset adlari) yok.
- **Pil**: ~~bilinmiyordu~~ — **COZULDU**: `0x0301` + (deger+1)*10 (yukariya bak).

## HCI snoop yakalamasi (2026-08-11, telefon + Soundcore uygulamasi)

Kaynak: `tools/captures/` (bugreport icinden `btsnoop_hci.log.last`). Xiaomi/MediaTek cihaz; kayit HCI ACL seviyesinde, L2CAP PDU `[len u16-LE][cid u16-LE][payload]`, cid 0x3040 (host->cihaz) / 0x0041 (cihaz->host).

Telefon uygulamasinin tek oturumda gonderdigi komutlar (tamaminda ACK geldi):

- `0x0101` bos istek (cihaz bilgisi; 164 bayt yanit)
- `0x0105` bos istek -> 258 bayt yanit: "A3957" model, MAC `dce55cd9bf90`, "04.90" firmware (zengin cihaz bilgisi)
- `0x9403` bos istek -> 60 bayt durum blobu (`77e61a69` + 0xff'ler; bayt anlamlari bilinmiyor)
- `0x9710 [01]` -> ACK (uygulama baslangicinda; anlami bilinmiyor)
- `0x8105` bos -> ACK x2 (anlami bilinmiyor)
- `0x8106 ANC`: payload `[mod, 0x51, 0x00, 0x01, 0x00, 0x00, 0x00]` (telefon formati; bizim `[mod, 0x10, 0x00, 0x01, 0x00, 0x01]` da cihazda calisiyor)
- `0x8510 [01]` / `[00]` -> ACK + `0x017f [00]` bildirimi. Iki kez ON/OFF cifti yakalandi. **Game Mode / Spatial Audio adayi** (hangisi oldugu kullanici dinleme testi bekliyor)
- `0x8110 [01 01 01] / [01 01 02] / [01 01 00] / [00 01 00] / [01 01 00]` -> ACK + bazi yazimlardan sonra `0x0301` pil bildirimi. **EQ preset secim adayi**
- `0x8703` 114 bayt payload: `[preset-id, 00,00,00] + 10 band x2 + maske + 10 band 16-bit x2 + 00,00`. Preset id'ler 01/02/03/00 yakalandi. **EQ band ayari (dogrulandi gorunuyor: cihazda ACK aliyor, band degerleri preset'e gore degisiyor)**

Pil:

- `0x0301` bildirimi: 3 bayt `[sol][sag][kutu]`. Gozlemlenen degerler: `09 07 06` (telefon), `09 06 06` (PC probe, ~10 dk sonra -> sag kulaklik 07'den 06'ya dustu = canli pil verisi). Carpan/formul henuz dogrulanmadi (x10 vs (x+1)x10; kullanici uygulamadaki yuzdeyi soyleyecek)
- `0x0101` yaniti payload[2..3] = `[sol][sag]` (aynı degerler; `09 07` telefon, `09 06` PC)
- `0x0301` istek olarak gonderildiginde yanit: henuz olculmedi (cihaz erisilemez duruma gecti)

Bilinmeyenler:

- `0x017f [00]` bildirimi: 0x8510/0x8703/0x8110 yazimlarindan sonra geliyor; `[00]` sabit gorunuyor. Semantik bilinmiyor (durum yankisi adayi)

## Oturum davranisi

- Prota tek oturumda `0x0101 -> 0x8106` bir kez `0x800710DD` ile dustu; pek cok kez tekrar denendi ve tekrarlanmadi (flaky). Uygulama bu yuzden tasiyici hatasinda oturumu yeniden acip komutu bir kez daha deniyor (`Liberty5Device::command` retry, 1.5 sn gecikme). 4/4 gercek cihaz kosusunda baglan -> device-info -> 3x ANC sirasi basarili.

## Kurallar

- Dogrulanmamis byte degerleri profilde YOK; uygulama bunlari kilitli tutar.
- Game Mode/EQ icin sonraki adim: Android HCI snoop capture (Soundcore uygulamasinda degistir) veya APK statik analiz (jadx).
- Deneysel komut gondermek icin: `scanner write <cihaz> <komut-hex> <payload-hex> --force` (onay sorar).
