# mouse-noti — Design Spec

**Date**: 2026-05-13
**Status**: Draft, pending implementation plan

## 1. Overview

데스크톱 시스템 알림이 발생할 때 마우스 커서 위치에 시각적 인디케이터를 표시하는 macOS + Windows 데스크톱 앱. 사용자 단축키(키보드 또는 마우스 버튼) 입력 시 커서 중심의 라디얼 퀵메뉴를 띄우고, 메뉴 항목은 GUI 설정창에서 편집 가능.

### 1.1 Goals

- 시스템 알림을 감지하여 마우스 커서에 인디케이터 표시 (스타일 설정 가능)
- 글로벌 단축키(키보드/마우스)로 커서 중심 라디얼 메뉴 호출
- 라디얼 메뉴는 컨텍스트 인지(최근 알림이 있으면 알림 액션, 없으면 일반 런처)
- GUI 설정창에서 메뉴 아이템 추가/수정/삭제

### 1.2 Non-Goals

- Linux 지원
- 알림 내용 저장/검색 (영구 알림 히스토리)
- 클라우드 동기화
- 모바일 앱

### 1.3 Target Platforms

- macOS 13 (Ventura) 이상
- Windows 10 (1809+, 빌드 17763) 이상

## 2. Tech Stack

- **프레임워크**: Tauri 2.x
- **백엔드**: Rust (tokio 비동기 런타임)
- **프론트엔드**: React + TypeScript + Tailwind CSS (Webview)
- **빌드**: Vite (프론트), Cargo (백엔드), pnpm (패키지)
- **주요 크레이트**:
  - `tauri`, `tauri-plugin-global-shortcut`
  - `windows` (Windows API 바인딩)
  - `objc2`, `core-foundation`, `core-graphics` (macOS)
  - `rdev` (크로스플랫폼 마우스 버튼 후킹)
  - `serde`, `toml` (설정 직렬화)

## 3. Architecture

```
┌─────────────────────────────────────────────┐
│  Tauri App (Rust core + Webview UI)         │
├─────────────────────────────────────────────┤
│                                             │
│  ┌──────────────┐    ┌──────────────────┐  │
│  │ NotiListener │    │  HotkeyListener  │  │
│  │  (mac/win    │    │  (kb: global-    │  │
│  │   adapter)   │    │   shortcut,      │  │
│  └──────┬───────┘    │   mouse: rdev)   │  │
│         │            └────────┬─────────┘  │
│         │ NotiEvent           │ TriggerEvt │
│         ▼                     ▼            │
│  ┌──────────────────────────────────────┐  │
│  │         EventBus (Rust tokio)        │  │
│  └────┬───────────────────┬──────────────┘  │
│       │                   │                 │
│       ▼                   ▼                 │
│  ┌─────────┐         ┌──────────────┐      │
│  │ Overlay │         │ RadialMenu   │      │
│  │ Window  │         │ Window (v2)  │      │
│  └─────────┘         └──────┬───────┘      │
│                             │ ActionId     │
│                             ▼              │
│                      ┌──────────────┐      │
│                      │ ActionRunner │      │
│                      └──────────────┘      │
│                                             │
│  ┌──────────────────────────────────────┐  │
│  │  SettingsStore (TOML)  +  Settings   │  │
│  │  GUI (Webview, React)                │  │
│  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 3.1 Components

#### NotiListener
OS별 어댑터, 공통 트레이트:

```rust
trait NotificationSource: Send + Sync {
    fn start(&self, tx: broadcast::Sender<NotiEvent>) -> Result<()>;
    fn stop(&self);
    fn health(&self) -> SourceHealth;
}

struct NotiEvent {
    app_id: String,
    app_name: String,
    title: String,
    body: String,
    icon: Option<Vec<u8>>,
    timestamp: SystemTime,
}
```

- **macOS**: `AXObserver` Accessibility API. Notification Center 프로세스(`NotificationCenter`)에 옵저버 등록, 자식 노드 추가 이벤트 캐치
- **Windows**: `Windows.UI.Notifications.Management.UserNotificationListener`, `NotificationChanged` 이벤트

#### HotkeyListener (v2)
- **키보드**: `tauri-plugin-global-shortcut` (Cmd/Ctrl+Shift+M 기본)
- **마우스**: `rdev::listen` 별도 스레드, 4/5번 버튼 또는 사용자 정의 조합
- 출력: `TriggerEvent { source, cursor_pos }`

#### EventBus
`tokio::sync::broadcast` 채널. NotiEvent, TriggerEvent, ConfigChanged 멀티캐스트. 최근 알림 5초 보존 (라디얼 메뉴 컨텍스트 결정용).

#### OverlayWindow
- 풀스크린 투명, always-on-top, click-through
- macOS: `NSWindow.ignoresMouseEvents = true`, `NSWindowCollectionBehaviorCanJoinAllSpaces`
- Windows: `WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOPMOST`
- 인디케이터 렌더: Canvas/SVG, 3종 스타일(링 펄스, 아이콘 배지, 지속 배지)
- 멀티 모니터: 커서 있는 디스플레이에 윈도우 재배치

#### RadialMenuWindow (v2)
- 작은 투명 윈도우, 단축키 시점 커서 좌표 중심
- N개 섹터(4~8), 호버 하이라이트, 마우스 릴리즈/클릭 시 실행
- ESC 또는 외부 클릭 시 취소
- 컨텍스트 결정: EventBus의 최근 5초 NotiEvent 조회 → 알림 모드 / 런처 모드

#### ActionRunner
액션 타입:
- `launch_app { path }` — 앱 실행
- `open_url { url }` — 브라우저 열기
- `run_shell { command, args, confirm }` — 셸 명령
- `noti_reply { text }` (Phase 3) — 마지막 알림에 응답

#### SettingsStore
- 경로:
  - macOS: `~/Library/Application Support/mouse-noti/config.toml`
  - Windows: `%APPDATA%\mouse-noti\config.toml`
- 백업: `config.toml.bak` (저장 전 이전 버전 보존)
- 변경 시 `ConfigChanged` 이벤트 발행 → 모든 컴포넌트 hot-reload

#### Settings GUI
별도 Tauri 윈도우, React + Tailwind:
- 인디케이터 스타일 선택
- 단축키 바인딩 (캡처 입력)
- 메뉴 아이템 리스트 (추가/수정/삭제/순서변경)
- 액션 타입별 입력 폼
- 라디얼 미리보기
- 권한 상태 패널

## 4. Data Flow

### 4.1 알림 → 인디케이터

```
OS Noti → NotiListener → NotiEvent → EventBus
       → OverlayWindow 구독자
       → 커서 좌표 조회 (mac: CGEventSourceCreate / win: GetCursorPos)
       → 인디케이터 렌더 (style 적용)
       → 애니메이션 종료 시 fade out
```

### 4.2 단축키 → 라디얼 (v2)

```
Hotkey press → HotkeyListener → TriggerEvent → EventBus
            → RadialMenuWindow.show_at(cursor_pos)
            → 최근 5초 알림 조회 → 컨텍스트 모드 결정
            → 사용자 섹터 선택 → ActionRunner.execute(action_id)
            → 윈도우 hide
```

### 4.3 설정 변경

```
Settings GUI → Tauri command (set_config)
            → SettingsStore.write(toml)
            → ConfigChanged 이벤트
            → 모든 컴포넌트 hot-reload
```

## 5. OS Permissions

### macOS
- **Accessibility API**: 시스템 환경설정 → 보안 및 개인 정보 보호 → 손쉬운 사용
- 첫 실행 시 권한 요청 다이얼로그 + 설정 페이지 자동 오픈
- 마우스 버튼 후킹도 동일 권한
- 배포 시 코드 사이닝 + notarization 필수

### Windows
- `UserNotificationListener.RequestAccessAsync()` 첫 실행 시
- 키/마우스 후킹은 별도 권한 불필요

권한 거부 시 트레이 아이콘 경고 뱃지 + 설정창 재요청 버튼.

## 6. Error Handling

| 시나리오 | 대응 |
|---|---|
| 권한 거부 | 트레이 경고 뱃지, 설정창 상태 + 재요청 |
| AX API 콜백 끊김 | 5초 헬스체크, 60초 무이벤트 시 옵저버 재등록, 3회 실패 시 사용자 알림 |
| 알림 폭주 | 같은 앱 1초 디바운스, 인디케이터 동시 1개 |
| 커서 좌표 실패 | 마지막 좌표 사용, 없으면 스킵 |
| 단축키 충돌 | 등록 실패 감지 → 설정창 빨간 표시 |
| Action 실행 실패 | 토스트 + 로그, 라디얼은 정상 닫힘 |
| 설정 파일 손상 | `.bak` 로드, 실패 시 기본값 + 알림 |
| 디스플레이 변경 | 디스플레이 이벤트 수신 → 윈도우 재생성 |

## 7. Testing Strategy

### 7.1 Unit (Rust)
- `SettingsStore` 직렬화/역직렬화
- `EventBus` throttle/디바운스
- `ActionRunner` 액션 파싱 (모킹 핸들러)
- `NotificationSource` 트레이트 모킹 → OverlayWindow 트리거 검증

### 7.2 Integration
- macOS: `cargo test --features mock-os` 가짜 알림 주입
- Windows: GH Actions Windows 러너에서 `UserNotificationListener` 호출
- OS API 통합은 로컬 매뉴얼 체크리스트

### 7.3 E2E (수동)
- OS별 알림 트리거 → 인디케이터 확인
- 단축키 → 라디얼 메뉴
- 권한 거부 흐름
- 멀티 모니터, DPI 변경, 다크모드 시각 회귀

### 7.4 Frontend
- Vitest + React Testing Library
- Tauri command 모킹

## 8. Roadmap

### Phase 0 — Scaffolding
- Tauri 프로젝트 초기화 (React+TS+Tailwind)
- 모노레포: `src-tauri/`, `src/`
- CI: GH Actions mac + win 매트릭스
- 트레이 아이콘 + 종료 메뉴
- **완료 기준**: 빈 앱이 양 OS 빌드/실행

### Phase 1 — MVP (인디케이터)
1. **알림 후킹 PoC**: Win `UserNotificationListener`, mac `AXObserver`, 공통 트레이트
2. **오버레이 윈도우**: 풀스크린 투명 click-through, 멀티 모니터, 인디케이터 3종(링 펄스/아이콘 배지/지속 배지)
3. **설정창**: 스타일 선택, 권한 상태, 자동 시작
4. **배포**: macOS DMG (sign+notarize), Windows MSI

**완료 기준**: 양 OS에서 시스템 알림 → 커서 인디케이터, 3종 스타일 전환, 권한 흐름 동작

### Phase 2 — 라디얼 메뉴 + 편집
1. **단축키 후킹**: 키보드(`global-shortcut`) + 마우스(`rdev`), 충돌 감지
2. **라디얼 UI**: 투명 윈도우, N섹터 호버, ESC 취소, 컨텍스트 감지
3. **ActionRunner**: launch_app / open_url / run_shell (셸은 확인 다이얼로그 옵션)
4. **편집 GUI**: 아이템 리스트, 액션 타입별 폼, 아이콘 선택, 미리보기

**완료 기준**: 단축키로 메뉴 호출, 컨텍스트 모드 전환, 설정창 편집 즉시 반영

### Phase 3 — Polish (옵션)
- 알림 응답 액션 (mac: AX 시뮬레이션, win: `ToastActivated`)
- 아이템별 직접 핫키
- 사용 통계 자동 정렬
- 테마 + 애니메이션 커스터마이즈

## 9. Risks

| 리스크 | 영향 | 완화 |
|---|---|---|
| macOS Accessibility API가 OS 업데이트로 깨짐 | 알림 감지 불가 | 헬스체크 + 재등록 로직, 정기 OS 베타 테스트 |
| 코드 사이닝 비용 (Apple Developer $99/yr) | 배포 지연 | 초기 unsigned 개발 빌드 배포, 정식 릴리스 시 사이닝 |
| `rdev` 마우스 후킹 안정성 | 단축키 불안정 | 폴백으로 키보드 단축키만 활성화 옵션 |
| 사용자가 권한 부여 안 함 | 핵심 기능 불가 | 명확한 권한 안내 + 재요청 UI, 알림 라우터 폴백 모드(v3) |
| 멀티 모니터 + 스케일링 좌표 버그 | 인디케이터 잘못된 위치 | OS별 좌표 정규화 유틸, 디스플레이 변경 이벤트 핸들링 |
