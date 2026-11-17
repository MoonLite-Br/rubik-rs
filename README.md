# Rubik (Rust) — học giải Rubik trên Android

Ứng dụng Android viết 100% bằng Rust, không XML/CSS/WebView, không GL driver. Giao diện, chữ
(fontdue) và khối Rubik 3D đều do một bộ vẽ phần mềm nhỏ tự render thẳng vào `ANativeWindow`.

* Ngôn ngữ: **Tiếng Việt / English** (theo hệ thống hoặc chọn tay)
* Giao diện: **Sáng / Tối / Theo hệ thống** (đọc cờ night-mode của Android, đổi ngay khi hệ thống đổi)
* 8 bài: Ký hiệu → Dấu cộng trắng → Góc trắng → Tầng giữa (F2L) → Dấu cộng vàng → Mặt vàng (OLL) → Đặt góc / Đặt cạnh (PLL)
* Trình phát từng bước (▶ ⏸ ⏮ ⏭) + chế độ **Luyện tập**: bấm 18 nước, xáo trộn, hoàn tác, theo dõi tiến độ (Cross → Layer 1 → F2L → OLL → Solved)

## Cấu trúc

```
Cargo.toml                 opt-level="z", lto, panic=abort, metadata cho cargo-apk
assets/fonts/              Poppins, Playfair Display, JetBrains Mono, Material Icons (nhúng bằng include_bytes!)
src/cube/                  mô hình khối: geom (hình học), moves (ký hiệu), state ([u8;54]), stage (nhận diện tiến độ)
src/tutorial/              lessons (dữ liệu song ngữ + thuật toán), player (máy trạng thái từng bước)
src/gfx/                   canvas (rasteriser AA), text (fontdue + cache glyph), cube3d (khối 3D + hoạt ảnh)
src/app/                   ui (widget immediate-mode), screens (4 màn hình + tab bar), settings
src/platform/android.rs    android-activity + ANativeWindow + AConfiguration
tests/engine.rs            cargo test: engine, bài học, font, vẽ mọi màn hình
examples/preview.rs        xuất ảnh .ppm các màn hình để xem trên máy tính, không cần emulator
```

Mô hình khối chỉ định nghĩa **một lần** bằng hình học (mỗi sticker = vị trí + pháp tuyến, mỗi nước đi = phép quay 90°),
từ đó sinh ra cả hoán vị 54 sticker lẫn hoạt ảnh 3D nên hai thứ không bao giờ lệch nhau. Hỗ trợ U D L R F B, M E S, x y z, u r f d l b / Rw.

## Build

```bash
cargo test                          # trên máy tính / Termux (không cần NDK)
cargo run --example preview         # xuất preview-*.ppm
# APK (cần Android SDK + NDK, hoặc dùng GitHub Actions có sẵn trong .github/workflows/android.yml)
rustup target add aarch64-linux-android
cargo install cargo-apk --locked
export ANDROID_HOME=... ANDROID_NDK_ROOT=...
cargo apk build --release           # -> target/release/apk/Rubik.apk
```

Release cần keystore: đặt `CARGO_APK_RELEASE_KEYSTORE` và `CARGO_APK_RELEASE_KEYSTORE_PASSWORD` (workflow CI tự tạo một cái).

## API hệ thống Android đang dùng (thuần NDK, không JNI)

| Việc | Cách làm |
|---|---|
| Nền sáng/tối/hệ thống | `AConfiguration` uiModeNight, đọc lại ở mỗi `ConfigChanged` (manifest khai báo `uiMode` nên activity không bị tạo lại) |
| Ngôn ngữ hệ thống | `AConfiguration` language (`vi` → Tiếng Việt, còn lại → English) |
| Thanh trạng thái / điều hướng | content rect của cửa sổ → insets |
| Lưu cài đặt | thư mục dữ liệu riêng của app (`settings.txt`) |
| Vòng đời | không vẽ khi `Pause` / mất cửa sổ; lúc rảnh chỉ thức dậy 4 lần/giây |
| Nút Back | đóng bài học / về tab Học trước khi thoát app |
| Thiếu RAM | `LowMemory` → xả cache glyph |

## RAM / tốc độ

* Vẽ trực tiếp vào buffer của cửa sổ (không có framebuffer thứ hai), tối đa 1080 px chiều ngang, hệ thống tự phóng to.
* Không nạp EGL/GLES nên không phải gánh vài chục MB của driver GPU.
* Font được parse **lười** (chỉ font màn hình đang dùng) và glyph được cache.
* Chỉ vẽ lại khi có thay đổi hoặc đang chạy hoạt ảnh.
* `opt-level = "z"` là theo yêu cầu; vòng lặp vẽ pixel nhạy với tối ưu hoá, nếu máy yếu thấy giật hãy thử `"s"` hoặc `3`.

## Lưu ý về font

`Poppins-Regular.ttf` **thiếu 88 glyph tiếng Việt** (Ơ ơ Ư ư và mọi chữ có dấu nặng / hỏi / mũ-dấu kép, ví dụ ệ ậ ợ ự ỉ).
Vì vậy khi giao diện là tiếng Việt, chữ thân bài dùng Playfair Display (đủ tiếng Việt); tiếng Anh dùng Poppins.
Muốn giao diện tiếng Việt cũng là sans-serif: bỏ font như *Be Vietnam Pro* hoặc *Noto Sans* vào `assets/fonts/` rồi đổi
hằng `SERIF_BODY` trong `src/gfx/text.rs` (test `text_has_no_missing_glyphs...` sẽ báo nếu còn thiếu glyph).

Giấy phép font: Playfair Display & JetBrains Mono & Poppins — SIL OFL 1.1; Material Icons — Apache 2.0.
