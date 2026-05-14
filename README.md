# Glance

[![CI](https://github.com/NacreousCloud/Glance/actions/workflows/ci.yml/badge.svg)](https://github.com/NacreousCloud/Glance/actions/workflows/ci.yml)
[![Release](https://github.com/NacreousCloud/Glance/actions/workflows/release.yml/badge.svg)](https://github.com/NacreousCloud/Glance/actions/workflows/release.yml)

데스크톱 알림이 발생할 때 마우스 커서 위치에 시각적 인디케이터를 표시하고, 커서 중심에서 라디얼 퀵메뉴를 띄우는 macOS + Windows 앱.

이전 이름: `mouse-noti` (v0.4.0부터 Glance로 변경).

## 현재 상태

**v0.4.0** — Phase 1 (알림 인디케이터) + Phase 2 (라디얼 퀵메뉴) 출시. 프로젝트 이름 `mouse-noti` → `Glance` 변경.

- 알림 인디케이터 3종 (Ring Pulse / Icon Badge / Persistent Badge)
- 라디얼 퀵메뉴 (커서 중심 부채꼴, 키보드 단축키 호출)
- 메뉴 액션: 앱 실행 / URL / 셸 명령
- 셸 명령 프리셋 16종 + 고급 수동 입력 모드
- 라디얼 외관 커스터마이즈 (색상, 투명도)
- 실행 실패 시 빨간 배지 알림 + Recent errors 패널

지원 OS: macOS 13+, Windows 10 (1809+).

---

## 사전 준비

### 공통

- **pnpm 9 이상** (`corepack enable && corepack prepare pnpm@latest --activate`)
- **Rust (stable)** (https://rustup.rs)
- **Node.js 20+**

### macOS

- Xcode Command Line Tools: `xcode-select --install`
- **권한**: 알림 인디케이터 자체는 CGWindowList 폴링으로 동작하므로 Accessibility 권한 불필요. Screen Recording 권한도 불필요 (window title은 읽지 않음). 첫 실행 시 권한 다이얼로그가 뜨더라도 알림 인디케이터 동작과는 무관.
  - Accessibility는 향후 Phase 2 (라디얼 메뉴 단축키, 추가 hooking)에서 사용 예정. 미리 부여해두면 좋음:
    - 시스템 설정 → 개인정보 보호 및 보안 → 손쉬운 사용 → Glance 토글 ON.
    - 권한 변경 후 앱 재시작 필요.

### Windows

- Visual Studio Build Tools (C++ 워크로드)
- WebView2 Runtime (Windows 11은 기본 포함)
- 첫 실행 시 **Notification Listener** 동의 프롬프트가 뜸. "허용"을 눌러야 다른 앱(Slack/Teams/Mail 등)의 토스트 알림을 감지함.

---

## 설치

저장소 루트에서:

```bash
pnpm install
```

Rust 의존성은 첫 `tauri dev`/`tauri build` 실행 시 자동으로 컴파일됨 (약 3~5분 소요, 이후엔 캐시됨).

---

## 실행

### 1. 개발 모드 (가장 흔함)

```bash
pnpm tauri dev
```

다음이 일어남:
1. Vite 프론트엔드 dev 서버 부팅
2. Rust 백엔드 컴파일
3. 트레이 아이콘이 menubar(macOS) / system tray(Windows)에 표시됨
4. macOS는 첫 실행 시 Accessibility 권한 다이얼로그 등장

종료: 트레이 아이콘 우클릭 → `Quit Glance`.

### 2. mock-os 기능 켜고 개발 모드

```bash
pnpm tauri dev --features mock-os
```

OS 권한 없이도 인디케이터 동작을 테스트할 수 있는 `inject_mock_event` Tauri 커맨드가 활성화됨.

테스트 방법:
1. 위 명령어로 실행
2. 트레이 → `Settings…` 으로 설정창 열기
3. 설정창에서 우클릭 → **Inspect Element** (DevTools 열기)
4. Console 탭에서:

```js
await window.__TAURI__.core.invoke('inject_mock_event', {
  appId: 'test',
  appName: 'Test App',
  title: 'Hello',
  body: 'world'
});
```

5. 현재 마우스 커서 위치에 인디케이터가 잠깐 표시되어야 함

### 3. 릴리스 빌드

```bash
pnpm tauri build              # DMG (macOS) / MSI (Windows) 생성
pnpm tauri build --no-bundle  # 바이너리만 (번들 스킵, 빠른 반복 테스트용)
pnpm tauri build --debug      # 디버그 심볼 포함 빌드
```

산출물 위치:
- 바이너리: `src-tauri/target/release/glance`
- macOS DMG: `src-tauri/target/release/bundle/dmg/*.dmg`
- Windows MSI: `src-tauri/target/release/bundle/msi/*.msi`

코드 사이닝은 환경 변수 (`APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` 등)로 설정. 미설정 시 unsigned 빌드.

### 4. 빌드한 바이너리 직접 실행

릴리스 바이너리를 dev 서버 없이 직접 띄울 때:

```bash
# 기존 인스턴스 종료
pkill -f "target/release/glance"

# 포그라운드 (로그 보임)
./src-tauri/target/release/glance

# 백그라운드 + 로그 파일
./src-tauri/target/release/glance > /tmp/glance.log 2>&1 &

# 로그 레벨 조정
RUST_LOG=glance=debug ./src-tauri/target/release/glance
```

dev 모드와 다른 점:
- HMR 없음. 프론트엔드 변경 시 `pnpm tauri build --no-bundle` 재실행.
- 첫 실행 시 권한 다이얼로그가 dev 빌드와 별도 항목으로 뜸 (바이너리 해시가 다름).
- 트레이 아이콘 우클릭 → `Quit Glance` 로 종료.

### 5. 변경된 백엔드 즉시 반영하는 가장 빠른 흐름

Rust 코드만 수정 후:

```bash
pnpm tauri build --no-bundle && \
  pkill -f "target/release/glance"; \
  ./src-tauri/target/release/glance > /tmp/glance.log 2>&1 &
```

빌드 실패 시 `pkill` 안 함 → 기존 인스턴스 유지. 빌드 성공 시 교체.

---

## 사용법

### 인디케이터 스타일 변경

1. 트레이 → `Settings…`
2. **Indicator style** 섹션에서 라디오 버튼 선택
   - **Ring Pulse**: 커서 주변에 초록 링이 짧게 펄스 (기본값)
   - **Icon Badge**: 커서 옆에 앱 이름 첫 글자가 적힌 배지가 잠깐 표시
   - **Persistent Badge**: 앱 이름이 적힌 배지가 5초간 유지

설정은 즉시 저장됨:

- macOS: `~/Library/Application Support/dev.glance.glance/config.toml`
- Windows: `%APPDATA%\dev\glance\glance\config\config.toml`

v0.3.x (mouse-noti) 시절 config는 자동으로 fallback 로드 → 첫 저장 시 새 경로로 이전.

### 권한 상태 확인

설정창 상단의 **Permissions** 패널이 현재 OS 권한 상태를 표시. "Required" 표시 시 **Grant** 버튼으로 재요청.

### 자동 시작

설정창의 **Launch at login** 체크박스. macOS는 LaunchAgent, Windows는 레지스트리 항목 자동 등록.

### 라디얼 퀵메뉴

마우스 커서 중심에 등장하는 부채꼴 메뉴. 등록한 단축키로 호출. 섹터를 호버 → 클릭하면 액션 실행.

#### 액션 종류

- **`launch_app`** — `.app` 번들 실행 (macOS `open` 사용). `Browse…` 버튼으로 파일 선택 가능.
- **`open_url`** — 기본 브라우저로 URL 열기.
- **`run_shell`** — 셸 명령 실행. 아이템별 `confirm` 옵션 (기본: true) → 실행 전 확인 다이얼로그.

#### 셸 명령 프리셋

`run_shell` 액션 추가 시 16개의 프리셋 그리드 노출 (스크린샷, 클립보드 처리, Wi-Fi 토글 등). 클릭 한 번으로 명령 채워짐.

직접 명령 입력하려면 **Advanced (manual)** 토글 → 명령/인자 자유 입력. 보안 위험 있으므로 신뢰하는 명령만.

#### 아이콘

- **이모지**: 40개 프리셋 그리드 또는 직접 텍스트 입력
- **앱 아이콘**: `launch_app` 액션 한정. `Use app icon` 버튼 → 시스템 앱 아이콘 추출 → base64 PNG 저장

#### 단축키 등록

Settings → Hotkey bindings → `+ Add binding`:

- **Keyboard**: `Capture` 버튼 → 원하는 키 조합 누름 (예: `Cmd+Shift+M`, `F13`, `F19`)
  - macOS는 `F20`–`F24` 미지원 (RegisterEventHotKey 제약). Karabiner로 재매핑 권장.
- **Mouse**: macOS는 현재 비활성화 (rdev 충돌). 키보드만 동작.
- **menu_mode**:
  - `all` — 전체 메뉴 아이템
  - `launcher` — `launcher` 태그
  - `notification` — `notification` 태그 + 라디얼 중앙에 최근 5초 알림 sender 이름

#### 닫힘 조건

- 섹터 클릭 → 액션 실행 후 자동 닫힘
- 중앙 disc 클릭 → 취소 (액션 없음)
- 메뉴 영역 밖 클릭 → 포커스 잃음 → 자동 닫힘
- **ESC** 키
- Settings → **Close radial on cursor leave** 활성화 시 커서가 메뉴 영역 벗어나면 즉시 닫힘

#### 외관 커스터마이즈

Settings → **Radial appearance** 섹션:

- **Backdrop** — 배경 사각형 영역 색/투명도. 기본값 0% (사각형 안 보임).
- **Sector** — 일반 섹터 색/투명도
- **Hover** — 마우스 올린 섹터 색
- **Center disc** — 중앙 취소 버튼 색

`Reset` 버튼으로 기본값 복원.

#### 보안 주의

- `run_shell` 액션은 Glance 프로세스 권한으로 실행. `confirm` 옵션 켜두기 권장.
- 스크린샷 (`screencapture`) 같은 화면 캡처 명령은 macOS Screen Recording 권한이 필요. 권한 부여 후 앱 재시작.

### 에러 알림

메뉴 액션이 실패하면 (예: 명령어 오타, 파일 없음):

1. 커서 위치에 **빨간 배지** 1줄 노출 — 에러 메시지 첫 줄 (80자)
2. Settings → **Recent errors** 패널에 시간순으로 누적 (최근 50건)
3. 각 항목 우측 **More** 버튼 → 전체 에러 메시지
4. **Clear** 버튼 → 로그 비우기

### 스크린샷 팁

라디얼에서 `screencapture -i` 같은 캡처 명령 실행 시, 메뉴가 자동으로 먼저 사라진 다음 캡처가 진행됨 (메뉴가 캡처 결과에 포함되지 않음).

---

## 테스트

```bash
pnpm test:run                                          # 프론트엔드 (Vitest, 10 tests)
cargo test --manifest-path src-tauri/Cargo.toml --lib  # Rust 유닛 (22 tests)
```

매뉴얼 QA 체크리스트: `docs/superpowers/checklists/mvp-manual-qa.md`

---

## 트러블슈팅

### macOS: 알림이 와도 인디케이터가 안 뜸

- Accessibility 권한 확인 (시스템 설정 → 개인정보 보호 및 보안 → 손쉬운 사용)
- 권한 부여 후에도 안 되면 앱을 완전히 종료 후 재실행 (`Quit Glance` 후 다시 실행)
- 권한 토글을 OFF → ON으로 한 번 더 끄고 켜기 (macOS 캐시 이슈)

### Windows: PermissionPanel이 계속 "Required" 표시

- Windows 10 1809 (빌드 17763) 이상이 필요함
- 알림 → 시스템 설정 → 시스템 → 알림 → 본인 앱 알림 활성화 확인

### Rust 컴파일 에러 (objc2 / windows 크레이트)

```bash
cargo clean --manifest-path src-tauri/Cargo.toml
pnpm tauri dev
```

캐시 충돌 시 종종 해결됨.

### Tauri CLI 못 찾음

```bash
pnpm install
```

`@tauri-apps/cli` 가 devDependencies에 있어야 함. `package.json` 확인.

---

## 알려진 제약 (MVP)

- **macOS mixed-DPI**: Retina 노트북 + 외부 non-Retina 모니터 조합에서 외부 모니터 인디케이터 위치가 부정확. 메인 디스플레이 스케일을 기준으로 계산하기 때문. 향후 fix 예정.
- **macOS 알림 텍스트 누락**: 인디케이터는 banner 윈도우 등장 시점을 즉시 감지하지만, banner의 title/body 텍스트는 비어있음. Screen Recording 권한을 요구하지 않기 위해 의도된 동작. Icon Badge / Persistent Badge 스타일은 sender (예: "nbagent", "NotificationCenter")의 첫 글자 또는 이름을 표시.
- **macOS DND/Focus 차단 알림**: banner UI가 표시되지 않은 알림(do-not-disturb, focus mode 차단 등)은 인디케이터도 발생하지 않음. CGWindowList polling은 banner 표시 시점을 기준으로 함.
- **macOS candidate 호스트 변경**: nbagent / NotificationCenter / UserNotificationCenter 세 프로세스를 폴링. macOS 메이저 업데이트로 banner 호스트 프로세스가 바뀌면 추가 필요 (`CANDIDATE_BUNDLE_IDS` / `CANDIDATE_EXEC_NAMES` 상수).

---

## 프로젝트 구조

```
glance/
├── src-tauri/                # Rust 백엔드 (Tauri)
│   ├── src/
│   │   ├── lib.rs            # 진입점, 빌더 체인
│   │   ├── tray.rs           # 트레이 아이콘
│   │   ├── settings.rs       # TOML 영속화
│   │   ├── event_bus.rs      # 디바운스 + recent window
│   │   ├── commands.rs       # Tauri commands (settings/permission)
│   │   ├── noti/             # OS별 알림 소스 + 트레이트
│   │   └── overlay/          # 오버레이 윈도우 + 좌표 helper
│   └── tauri.conf.json
├── src/                      # React 프론트엔드
│   ├── overlay/              # 인디케이터 컴포넌트 (3종)
│   └── settings/             # Settings GUI
└── docs/superpowers/         # 스펙 + 플랜 + QA 체크리스트
```

---

## 문서

- 디자인 스펙: `docs/superpowers/specs/2026-05-13-mouse-noti-design.md`
- MVP 구현 플랜: `docs/superpowers/plans/2026-05-13-mouse-noti-mvp.md`
- 매뉴얼 QA 체크리스트: `docs/superpowers/checklists/mvp-manual-qa.md`

---

## 다음 단계

- macOS mixed-DPI 인디케이터 위치 정확도 개선
- macOS 마우스 단축키 (NSEvent global monitor 기반 재구현, 현재 비활성)
- Windows 라디얼 메뉴 + 알림 소스 정합성 점검
- 자동 업데이트 (tauri-plugin-updater) — 배포 대상 늘어나면
