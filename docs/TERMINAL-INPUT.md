# Terminal input ownership

## English

The common terminal Kit converts pixel, line, and page deltas into integral cell steps. It also
owns ordinary history scrollback. This provider receives only the two PTY-producing routes:
mouse reports and DEC mode 1007 alternate scroll. There is no generic mouse encoder or scrollback
fallback in this repository.

For a mouse-report route, the adapter takes a fresh engine mode snapshot and refuses the request
with `WHEEL_MODE_CHANGED` when live click, drag, or motion reporting is no longer active. Each
vertical step becomes live-engine button 4 or 5 input and each horizontal step becomes button 6 or
7 input. The provider ABI then applies its current legacy, UTF-8, SGR, or URXVT encoding, cell
position, and Shift/Alt/Control modifier rules. Repetition produces one native report per integral
step. Meta has no bit in these mouse protocols and therefore does not alter a report.

For alternate scroll, the adapter requires an active alternate screen, DEC mode 1007, and no
higher-priority mouse-reporting route. Vertical and horizontal steps become cursor keys with the
engine's live normal/application cursor encoding. This route follows the engine's wheel behavior
and does not add modifier bytes. If those live facts change after the common Kit selected the
route, the adapter refuses the stale route instead of reinterpreting it.

Two engine facts are deliberately not aliased. DEC 9 X10 tracking and DEC 1001 highlight tracking
have no distinct fields in the current public terminal-mode snapshot, so the common Kit cannot yet
select a mouse-report wheel route for them. The ignored
`red_x10_and_highlight_modes_can_generate_a_mouse_report_route` test names that contract gap. The
engine also supports pixel-coordinate SGR reports, but `EngineWheelInput` supplies cell coordinates;
this adapter therefore makes no pixel-coordinate wheel claim.

## 한국어

공통 터미널 Kit은 픽셀·줄·페이지 단위 델타를 셀 단위 정수 스텝으로 누적하고 일반 히스토리
스크롤백을 소유한다. 이 provider가 받는 PTY 출력 경로는 마우스 리포트와 DEC 1007 대체 화면
스크롤뿐이다. 이 저장소에는 범용 마우스 인코더나 스크롤백 fallback이 없다.

마우스 리포트 경로에서는 엔진 모드를 다시 읽고, 현재 click·drag·motion reporting이 꺼졌다면
`WHEEL_MODE_CHANGED`로 요청을 거부한다. 세로 스텝은 live engine의 버튼 4/5, 가로 스텝은 버튼
6/7 입력이 된다. 그 뒤 provider ABI가 현재 legacy·UTF-8·SGR·URXVT 인코딩, 셀 위치,
Shift/Alt/Control modifier 규칙을 적용한다. 반복 입력은 정수 스텝마다 native report 하나를
만든다. Meta는 이 마우스 프로토콜에 대응하는 비트가 없으므로 report를 바꾸지 않는다.

대체 스크롤 경로는 대체 화면, DEC 1007, 그리고 더 우선하는 마우스 리포트 경로가 없다는 조건을
모두 요구한다. 세로·가로 스텝은 엔진의 현재 normal/application cursor 인코딩을 따르는 방향키가
된다. 이 경로는 엔진의 wheel 동작과 동일하게 modifier byte를 추가하지 않는다. 공통 Kit이 경로를
정한 뒤 이 상태가 바뀌면 다른 경로로 재해석하지 않고 stale route를 거부한다.

두 엔진 상태는 의도적으로 alias하지 않는다. DEC 9 X10 tracking과 DEC 1001 highlight tracking은
현재 공개 터미널 모드 snapshot에 별도 필드가 없어 공통 Kit이 마우스 리포트 wheel 경로를 선택할
수 없다. 무시된 `red_x10_and_highlight_modes_can_generate_a_mouse_report_route` 테스트가 이 계약
공백을 이름으로 남긴다. 엔진은 pixel-coordinate SGR report도 지원하지만 `EngineWheelInput`은 셀
좌표만 제공하므로, 이 adapter는 pixel-coordinate wheel 지원을 주장하지 않는다.
