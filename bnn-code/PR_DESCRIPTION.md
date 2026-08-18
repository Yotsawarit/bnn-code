# fix(inference): replace panics with graceful error handling in model.rs

## ปัญหาที่แก้ไข

`src/inference/model.rs` เดิมใช้ `unwrap()` / `expect()` ในทุกขั้นตอนของการโหลด model
และการ inference ทำให้โปรแกรม **panic และ crash** แทนที่จะแสดง error message ที่อ่านเข้าใจได้

กรณีที่พบบ่อย:
- ไฟล์ model ไม่มี (ยังไม่ได้รัน `download_model.sh`)
- download model ค้างกลางทาง ไฟล์เสีย
- รันบน CPU รุ่นเก่าที่ไม่รองรับ SSE4.2

## สิ่งที่เปลี่ยนแปลง

### 1. `BnnError` enum (`thiserror`)
แทน `unwrap()` ทั้งหมดด้วย `Result<_, BnnError>` ครอบคลุม 8 กรณี:

| Variant | เกิดเมื่อ |
|---------|----------|
| `ModelNotFound` | ไฟล์ไม่มี — พร้อม hint ให้รัน `download_model.sh` |
| `ModelTooSmall` | ไฟล์ < 64 bytes — แจ้งว่า download ค้าง |
| `InvalidModelFormat` | magic byte ผิด — ไม่ใช่ ONNX protobuf |
| `CpuNotSupported` | ไม่มี SSE4.2 — บอก command ที่ตรวจได้ |
| `RuntimeInit` | ONNX Runtime init ล้มเหลว |
| `SessionLoad` | โหลด session ล้มเหลว (opset ผิด ฯลฯ) |
| `InferenceFailed` | forward pass ล้มเหลว |
| `OutputShapeMismatch` | output tensor shape ผิดคาด |

### 2. CPU SSE4.2 check ก่อน load ONNX Runtime
ใช้ `is_x86_feature_detected!("sse4.2")` ตรวจสอบก่อนแตะ ONNX Runtime เลย
บน ARM (macOS Apple Silicon, Raspberry Pi) ข้าม check โดยอัตโนมัติ

### 3. Three-layer model file validation
ก่อนส่ง path ให้ ONNX Runtime:
1. ไฟล์ต้องมีอยู่ (จับ "ยังไม่ download")
2. ขนาด ≥ 64 bytes (จับ partial download)
3. first byte = `0x08` (ONNX protobuf magic)

### 4. `load_default()` convenience method
ใช้ path เดียวกับที่ `scripts/download_model.sh` เขียน (`~/.cache/bnn-code/models/codeberta-small.onnx`)
ทำให้ผู้ใช้ที่รัน download script แล้วไม่ต้องระบุ path เอง

### 5. Structured logging (`tracing`)
แทน `eprintln!` ด้วย `tracing::error!/warn!/info!` เพื่อให้ log framework ของ project control ได้

## การทดสอบ

เพิ่ม 7 unit tests ที่รันได้โดยไม่ต้องมีไฟล์ `.onnx` จริง:

```
test tests::missing_file_gives_model_not_found      ... ok
test tests::partial_download_gives_too_small        ... ok
test tests::wrong_magic_gives_invalid_format        ... ok
test tests::correct_magic_passes_validation         ... ok
test tests::cpu_check_returns_without_panicking     ... ok
test tests::cpu_not_supported_message_is_actionable ... ok
test tests::default_model_path_contains_codeberta  ... ok
```

## Dependencies ที่เพิ่ม (Cargo.toml)

```toml
[dependencies]
thiserror = "1"
tracing   = "0.1"
dirs      = "5"          # สำหรับ default_model_path()

[dev-dependencies]
tempfile  = "3"
```

## ตัวอย่าง error message ก่อน/หลัง

**ก่อน (panic):**
```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value:
  OrtError(...)', src/inference/model.rs:47
```

**หลัง (graceful):**
```
Error: Model file not found: /home/user/.cache/bnn-code/models/codeberta-small.onnx
Hint: run `bash scripts/download_model.sh` to download CodeBERTa-small,
or `python3 scripts/download_model.py --model codeberta-small`
```
