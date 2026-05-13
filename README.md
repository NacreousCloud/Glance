# mouse-noti

데스크톱 알림이 발생할 때 마우스 커서 위치에 시각적 인디케이터를 표시하는 macOS + Windows 앱.

## 현재 상태

Phase 1 (MVP). 인디케이터 표시까지. 라디얼 퀵메뉴는 Phase 2 예정.

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
    - 시스템 설정 → 개인정보 보호 및 보안 → 손쉬운 사용 → mouse-noti 토글 ON.
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

종료: 트레이 아이콘 우클릭 → `Quit mouse-noti`.

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
pnpm tauri build --debug      # 디버그 심볼 포함 빌드
```

산출물 위치:
- macOS: `src-tauri/target/release/bundle/dmg/*.dmg`
- Windows: `src-tauri/target/release/bundle/msi/*.msi`

코드 사이닝은 환경 변수 (`APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` 등)로 설정. 미설정 시 unsigned 빌드.

---

## 사용법

### 인디케이터 스타일 변경

1. 트레이 → `Settings…`
2. **Indicator style** 섹션에서 라디오 버튼 선택
   - **Ring Pulse**: 커서 주변에 초록 링이 짧게 펄스 (기본값)
   - **Icon Badge**: 커서 옆에 앱 이름 첫 글자가 적힌 배지가 잠깐 표시
   - **Persistent Badge**: 앱 이름이 적힌 배지가 5초간 유지

설정은 즉시 저장됨 (`~/Library/Application Support/dev.mouse-noti.app/config.toml` on macOS, `%APPDATA%\dev.mouse-noti.app\config.toml` on Windows).

### 권한 상태 확인

설정창 상단의 **Permissions** 패널이 현재 OS 권한 상태를 표시. "Required" 표시 시 **Grant** 버튼으로 재요청.

### 자동 시작

설정창의 **Launch at login** 체크박스. macOS는 LaunchAgent, Windows는 레지스트리 항목 자동 등록.

---

## 테스트

```bash
pnpm test:run                                          # 프론트엔드 (Vitest, 6 tests)
cargo test --manifest-path src-tauri/Cargo.toml --lib  # Rust 유닛 (10 tests)
```

매뉴얼 QA 체크리스트: `docs/superpowers/checklists/mvp-manual-qa.md`

---

## 트러블슈팅

### macOS: 알림이 와도 인디케이터가 안 뜸

- Accessibility 권한 확인 (시스템 설정 → 개인정보 보호 및 보안 → 손쉬운 사용)
- 권한 부여 후에도 안 되면 앱을 완전히 종료 후 재실행 (`Quit mouse-noti` 후 다시 실행)
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
mouse-noti/
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

## 다음 단계 (Phase 2 예정)

- 키보드/마우스 단축키로 호출하는 라디얼 퀵메뉴
- 메뉴 아이템 편집 GUI (앱 실행 / URL / 셸 명령)
- 컨텍스트 인지 (최근 알림 있으면 알림 액션, 없으면 일반 런처)
