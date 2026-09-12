# 이미지 순서 기반 일괄 리네이밍 프로그램 개발 계획서

## 1. 프로젝트 개요

### 프로젝트명
가칭: **nekote**

### 목적

이미지 파일을 탐색기에서 하나씩 이름을 변경하는 번거로움을 해결한다.

특히 다음과 같은 작업을 GUI에서 직관적으로 수행할 수 있도록 한다.

- 이미지 파일을 썸네일로 확인
- 드래그 앤 드롭으로 순서 변경
- 새로운 이미지를 원하는 위치에 삽입
- 전체 이미지의 번호를 자동으로 재계산
- 변경 결과를 미리 확인
- 실제 파일명을 안전하게 일괄 변경
- 실수했을 경우 Undo

### 핵심 사용 예

기존:

```text
001.jpg
002.jpg
...
059.jpg
060.jpg
061.jpg
...
100.jpg
```

새 이미지 10장을 60번 위치에 삽입:

```text
001.jpg
002.jpg
...
059.jpg
[새 이미지 10장]
060.jpg
061.jpg
...
100.jpg
```

사용자가 "적용"하면:

```text
001.jpg
002.jpg
...
059.jpg

060.jpg  ← 새 이미지
061.jpg  ← 새 이미지
...
069.jpg  ← 새 이미지

070.jpg  ← 기존 060
071.jpg  ← 기존 061
...
110.jpg  ← 기존 100
```

으로 자동 변경한다.

---

# 2. 핵심 설계 원칙

## 2.1 파일명을 직접 편집하는 프로그램이 아니라 "순서를 편집하는 프로그램"

기존 일괄 리네이밍 프로그램은 보통

```text
파일명 → 규칙 적용 → 새로운 파일명
```

방식이다.

본 프로젝트에서는 반대로

```text
파일
 ↓
순서
 ↓
번호
 ↓
파일명
```

이라는 개념으로 설계한다.

사용자는 번호 자체를 조작하기보다 **이미지의 순서를 조작**한다.

---

## 2.2 원본 파일은 적용 버튼을 누르기 전까지 변경하지 않는다

GUI에서 아무리 드래그하고 정렬해도 실제 파일 시스템은 변경하지 않는다.

```text
GUI 상태
   ↓
사용자 확인
   ↓
Apply
   ↓
실제 파일 rename
```

이를 통해 실수로 파일명이 변경되는 것을 방지한다.

---

## 2.3 Rename은 안전한 Transaction 방식으로 처리

다음과 같은 문제가 발생할 수 있다.

```text
060.jpg → 070.jpg
061.jpg → 071.jpg
```

그런데 이미 `070.jpg`가 존재하면 충돌한다.

따라서 직접 덮어쓰지 않고 임시 이름을 거치는 방식으로 구현한다.

```text
060.jpg → .renamer_tmp_xxx
061.jpg → .renamer_tmp_yyy

...

.renamer_tmp_xxx → 070.jpg
.renamer_tmp_yyy → 071.jpg
```

또한 작업 전/후 상태를 기록하여 가능한 경우 Undo할 수 있도록 한다.

---

# 3. 플랫폼

1차 목표:

- Windows
- Linux
- macOS

가능한 한 동일한 코드베이스로 제공한다.

### 배포 형태

```text
Windows
  .exe / installer

Linux
  AppImage 또는 배포판별 패키지

macOS
  .app / .dmg
```

---

# 4. 기술 스택

## 핵심 언어

### Rust

선정 이유:

- 네이티브 성능
- 낮은 메모리 오버헤드
- Windows / Linux / macOS 지원
- 파일 시스템 작업에 적합
- 병렬 이미지 처리에 유리
- 단일 실행 파일 형태의 배포가 비교적 용이
- 메모리 안정성

---

## GUI

### Slint

목표:

- 크로스플랫폼 GUI
- 반응성 높은 UI
- Drag & Drop
- 썸네일 리스트
- 대량 이미지 표시


---

## 이미지 처리

초기에는 Rust 이미지 처리 라이브러리를 사용한다.

주요 기능:

- JPEG
- PNG
- WebP
- GIF 등의 이미지 탐색
- 썸네일 생성
- 이미지 크기 확인

추후 대량 이미지 처리 성능이 부족할 경우 libvips 계열을 검토한다.

---

# 5. 프로그램 구조

```text
┌─────────────────────────────┐
│            GUI              │
│                             │
│ Folder / Thumbnail / List   │
│ Drag & Drop / Preview       │
└──────────────┬──────────────┘
               │
               ↓
┌─────────────────────────────┐
│      Application Layer      │
│                             │
│ File Manager                │
│ Reorder Manager             │
│ Numbering Manager           │
│ Rename Manager              │
│ Undo Manager                │
└──────────────┬──────────────┘
               │
               ↓
┌─────────────────────────────┐
│       File System Layer     │
│                             │
│ Scan                        │
│ Read Metadata               │
│ Rename                      │
│ Move                        │
└─────────────────────────────┘
```

---

# 6. 데이터 모델

파일 하나를 단순히 파일명으로만 관리하지 않는다.

예:

```text
ImageFile

id
original_path
current_path
original_name
current_name
extension
width
height
thumbnail
position
```

실제 GUI에서는 다음과 같이 관리한다.

```text
ImageList

[Image A]
position = 0

[Image B]
position = 1

[Image C]
position = 2
```

사용자가 드래그하면 `position`만 변경한다.

최종 파일명은 position으로부터 계산한다.

예:

```text
position 0 → 001.jpg
position 1 → 002.jpg
position 2 → 003.jpg
```

---

# 7. 주요 기능

## Phase 1 — 기본 파일 탐색

목표:

> 폴더를 열고 이미지가 보인다.

구현:

- 폴더 선택
- 이미지 파일 탐색
- 파일 목록 표시
- 파일명 표시
- 이미지 썸네일 표시
- 이미지 정렬

지원 확장자:

```text
jpg
jpeg
png
webp
gif
bmp
```

---

# 8. Phase 2 — Drag & Drop

가장 중요한 핵심 기능.

예:

```text
[001] [002] [003] [004] [005]
```

사용자가 005를 002와 003 사이로 이동:

```text
[001] [002] [005] [003] [004]
```

GUI 내부의 순서가 변경된다.

이 단계에서는 실제 파일명을 변경하지 않는다.

---

# 9. Phase 3 — 자동 번호화

사용자가 원하는 numbering 규칙을 설정한다.

예:

```text
Start number: 1
Padding: 3
Step: 1
Extension: preserve
```

결과:

```text
001.jpg
002.jpg
003.jpg
...
```

설정 가능한 항목:

```text
시작 번호
자리수
증가값
확장자 유지
파일명 Prefix
파일명 Suffix
```

예:

```text
img_001.jpg
img_002.jpg
img_003.jpg
```

또는

```text
chapter_01.png
chapter_02.png
chapter_03.png
```

---

# 10. Phase 4 — Rename Preview

실제 적용 전에 변경 내용을 보여준다.

예:

```text
현재 파일       변경 후

060.jpg    →    070.jpg
061.jpg    →    071.jpg
062.jpg    →    072.jpg

new_a.jpg  →    060.jpg
new_b.jpg  →    061.jpg
new_c.jpg  →    062.jpg
```

사용자가 확인한 뒤:

```text
[Cancel]          [Apply]
```

Apply를 눌러야 실제 파일 시스템을 변경한다.

---

# 11. Phase 5 — 안전한 Rename

rename 작업을 하나의 작업 단위로 취급한다.

### 1단계

현재 상태 저장:

```text
old_name
new_name
```

### 2단계

모든 파일을 임시 이름으로 이동.

### 3단계

임시 이름을 최종 이름으로 변경.

### 4단계

성공 여부 확인.

### 5단계

작업 기록 저장.

---

# 12. Phase 6 — Undo

가장 중요한 편의 기능 중 하나.

예:

```text
작업 전

001.jpg
002.jpg
003.jpg

↓

작업 후

001.jpg
003.jpg
002.jpg
```

Undo:

```text
001.jpg
002.jpg
003.jpg
```

rename뿐 아니라 가능하면 순서 변경도 Undo 대상으로 관리한다.

---

# 13. Phase 7 — Thumbnail Cache

이미지가 수천~수만 장일 경우 모든 원본을 동시에 디코딩하지 않는다.

구조:

```text
원본 이미지
     ↓
thumbnail 생성
     ↓
memory cache
     ↓
disk cache
     ↓
GUI
```

화면에 필요한 이미지를 우선적으로 처리한다.

이를 통해 대량 이미지에서도 UI가 멈추지 않도록 한다.

---

# 14. Phase 8 — 대량 파일 최적화

목표:

```text
1,000장
10,000장
50,000장
```

에서도 사용할 수 있도록 한다.

고려사항:

- Lazy loading
- Virtualized list
- Thumbnail cache
- Background decoding
- 병렬 thumbnail 생성
- 파일 시스템 스캔 비동기화
- GUI thread와 작업 thread 분리

특히 GUI thread에서 이미지 디코딩을 하지 않는다.

---

# 15. 추가 기능 후보

MVP 이후 추가한다.

### 파일명 규칙

```text
prefix
suffix
날짜
원본 이름 유지
번호 삽입 위치
```

### 정렬

```text
파일명
생성일
수정일
파일 크기
이미지 해상도
EXIF 촬영 시간
```

### 필터

```text
JPG만
PNG만
가로 이미지
세로 이미지
특정 파일명
```

### 선택 기능

```text
Shift 선택
Ctrl 선택
다중 선택
선택 영역 이동
```

### 이미지 미리보기

파일을 클릭하면 큰 이미지로 표시.

---

# 16. MVP 범위

처음부터 모든 기능을 만들지 않는다.

MVP는 다음만 구현한다.

```text
[1] 폴더 열기
       ↓
[2] 이미지 목록 표시
       ↓
[3] 썸네일 표시
       ↓
[4] Drag & Drop
       ↓
[5] 번호 자동 계산
       ↓
[6] 변경 결과 Preview
       ↓
[7] 실제 Rename
```

이것만 완성해도 프로젝트의 핵심 목적은 달성된다.

---

# 18. 테스트

### 기능 테스트

```text
10개
100개
1,000개
10,000개
```

파일로 테스트한다.

### Rename 충돌

```text
001 → 002
002 → 003
003 → 001
```

같은 상황을 테스트한다.

### 예외 상황

```text
파일 삭제
파일 이동
읽기 권한 없음
파일명 충돌
디스크 공간 부족
프로그램 강제 종료
```

### 대량 이미지

```text
10,000장의 이미지
```

를 넣고 UI가 정상적으로 반응하는지 확인한다.

---

# 19. GitHub 프로젝트 구성

```text
nekote/
│
├── src/
│   ├── main.rs
│   ├── filesystem/
│   ├── thumbnail/
│   ├── reorder/
│   ├── rename/
│   └── undo/
│
├── ui/
│
├── tests/
│
├── assets/
│
├── Cargo.toml
├── README.md
├── LICENSE
└── CONTRIBUTING.md
```

README에는 다음을 포함한다.

```text
프로젝트 소개
스크린샷
주요 기능
사용 방법
설치 방법
지원 OS
성능
아키텍처
Rename 안전성
Known Issues
License
```

---

# 21. 최종 목표

최종 사용 경험은 다음과 같다.

```text
폴더를 연다
      ↓
이미지들이 썸네일로 나온다
      ↓
새 이미지를 원하는 위치에 드래그한다
      ↓
순서가 자동으로 재계산된다
      ↓
변경될 파일명을 확인한다
      ↓
Apply
      ↓
파일명이 안전하게 변경된다
```

사용자가 별도의 정규식이나 복잡한 rename 규칙을 이해하지 않아도 사용할 수 있는 것을 목표로 한다.

### 핵심 가치

**"파일명을 관리하는 프로그램이 아니라, 이미지의 순서를 관리하면 파일명이 알아서 따라오는 프로그램."**