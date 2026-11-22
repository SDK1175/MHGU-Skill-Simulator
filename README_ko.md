<p align="center">
  <a href="README.md">English</a> | <strong>한국어</strong>
</p>

# MHGU Skill Simulator (MHXX Recomp용 수정판)

[ZellD-0371님의 MHGU Skill Simulator](https://github.com/ZellD-0371/MHGU-Skill-Simulator)를 **[MHXX Recomp](https://github.com/SDK1175/MHXXRecomp)(몬스터헌터 더블크로스 일어판 PC 포팅) + 덥크 한글 패치**에서 쓰도록 고친 비공식 수정판입니다. 원작은 MIT 라이선스이며, 시뮬레이터 기능과 검색 엔진은 원작 그대로입니다.

- 영미판(MHGU)이나 MHGU Recomp에서 쓰실 분은 [원작](https://github.com/ZellD-0371/MHGU-Skill-Simulator/releases)을 받으세요.
- 이 수정판의 문제는 원작자가 아니라 [이 저장소의 이슈](https://github.com/SDK1175/mhgu-skill-simulator/issues)나 [MHXX Recomp 이슈](https://github.com/SDK1175/MHXXRecomp/issues)로 알려 주세요.

<p align="center"><a href="docs/screenshots.md">스크린샷 보기 (원작)</a></p>

원하는 스킬을 고르면, 실제로 가진 방어구·호석·장식품으로 그 스킬이 발동하는 조합을 찾아 줍니다. 가진 호석으로 만들 수 없으면 필요한 호석과, 감정 한 번에 그 호석이 나올 확률(추정)을 보여 줍니다.

## 필요한 것

- **MHXX Recomp 0.10.9 이상**: 이 모드에 필요한 기능(플러그인 메모리, 마우스·키 입력, 한글 패치 글자, 영문판 주소 변환)이 포트에 들어 있습니다. 그보다 오래된 포트에서는 로드되지 않거나 창이 열리지 않습니다.
- 덥크 한글 패치: 없어도 동작하지만, 메뉴의 "스킬 시뮬레이터" 이름은 한글 패치가 있을 때 한글로 나옵니다.

## 원작과 다른 점

- 덥크의 마을·집회소 메뉴를 알아보고 **메뉴 3페이지**에 "스킬 시뮬레이터"를 넣습니다 (원작은 영문판 메뉴만 알아봄).
- 메뉴 이름과 설명을 **덥크 한글 패치 글자**로 넣습니다 (원작은 한자로 판단해 중국어로 나옴).
- 덥크에서 캐릭터를 불러온 것을 제대로 알아봅니다 (원작은 덥크에서 "플레이 중"이 안 돼 창이 안 열림).
- devkitARM 없이 빌드하는 스크립트 `tools/build_xx.sh`.

영문판·에뮬레이터에서의 동작은 원작과 같습니다.

## 주요 기능 (원작 그대로)

- 목표 스킬 최대 8개로 검색, 장비 상자·호석·현재 장비를 게임에서 직접 읽음
- 가진 호석으로 안 될 때 미보유 호석까지 검색하고, 감정 1회당 확률(추정) 표시
- 검색 결과: 추천 세트, 남는 슬롯, 방어력, 속성 내성, 장식품 배치, 발동 스킬, 새로 만들 부위와 생산 소재·금액
- 같은 스킬이 나오는 부위별 대체 방어구, 현재 장비와 비교, 세트에 더 띄울 수 있는 추가 스킬
- 조건에 맞는 세트가 없으면 어떤 조건 하나를 바꾸면 찾을 수 있는지 알려 줌
- 9개 언어 (한국어 포함), Nintendo·Xbox·PlayStation 버튼 아이콘, 마우스·키보드 지원

## 설치

1. 게임을 종료합니다.
2. `MHGU-Skill-Simulator_vX.X.X-mhxx.zip`의 압축을 풉니다.
3. 압축을 푼 `MHGU-Skill-Simulator` 폴더를 MHXX Recomp의 `mods` 폴더에 넣습니다. (원작을 넣어 두셨다면 지우고 넣으세요)
4. 게임을 실행합니다. 런처의 모드 목록에서 꺼져 있으면 켭니다.

```text
mods/
└── MHGU-Skill-Simulator/
    └── romfs/
        └── nativeNX/
            └── plugins/
                ├── skill_simulator.nro
                └── skill-simulator/
                    ├── LICENSE.txt
                    ├── NOTO_LICENSE.txt
                    ├── THIRD_PARTY_NOTICES.md
                    ├── atlas.rgba
                    ├── font.otf
                    └── names.en.bin … names.es.bin
```

## 사용 방법

- 캐릭터를 불러온 뒤, 마을이나 집회소에서 **+ 메뉴를 열고 3페이지의 "스킬 시뮬레이터"**를 고릅니다.
- 퀘스트 중에는 메뉴에 없습니다. **Insert**로 Forge 설정을 열고 `Plugin UI > Skill Simulator`의 **열기** 버튼을 쓰세요.

## 설정

Insert로 Forge 설정을 열고 `Plugin UI > Skill Simulator`로 이동합니다.

| 설정 | 선택지 |
| --- | --- |
| Language | 지원하는 9개 언어 중 선택 (기본값 English) |
| Button layout | Nintendo / Xbox / PlayStation |
| 로그 기록 활성화 | 문제 보고용으로 `log.txt`에 기록 (기본값 꺼짐). 문제를 알릴 때 켜고 다시 해 본 뒤 로그를 첨부해 주세요 |

## 저장 데이터

설정과 캐릭터별 데이터는 게임의 현재 세이브 폴더 안에 별도 INI 파일로 저장됩니다.

| 파일 | 내용 |
| --- | --- |
| `skill_simulator.ini` | 언어, 버튼 배치, 로그 기록 |
| `skill_simulator_character_1.ini` | 첫 번째 캐릭터 슬롯의 마지막 조건, 저장한 세트, 제외한 방어구, 최근 스킬 |
| `skill_simulator_character_2.ini` | 두 번째 캐릭터 슬롯의 같은 내용 |
| `skill_simulator_character_3.ini` | 세 번째 캐릭터 슬롯의 같은 내용 |

MHXX Recomp의 기본 경로는 `%APPDATA%\MHXXRecomp\save\save_0\`입니다. 모드를 업데이트해도 이 파일들은 교체되지 않습니다.

## 참고

- 호석 확률은 호석 생성표로 계산한 추정값이며, 게임의 정확한 확률이 아닙니다.
- 창은 캐릭터를 불러온 뒤에만 열립니다.

## 소스에서 빌드하기 (MHXX Recomp용)

필요한 도구: MSYS2(UCRT64), `rust-src`가 포함된 Rust nightly, Python 3. devkitARM과 `cargo forge`는 필요 없습니다.

```bash
bash tools/build_xx.sh                    # 기본: /f/MHXX_port/MHXXRecomp/build/mods 에 설치
bash tools/build_xx.sh /c/게임/MHXXRecomp/mods   # 다른 mods 폴더
```

`dist/`에 `skill_simulator.nro`와 배포용 zip(`MHGU-Skill-Simulator_v<버전>-mhxx.zip`)이 만들어집니다. 원작의 빌드 방법(`tools/build_plugin.sh`, devkitARM)도 그대로 쓸 수 있습니다.

테스트:

```bash
cd engine && cargo test --release                 # 엔진 (전수 탐색 대조 포함)
cd plugin-hosttest && cargo run --release         # PC에서 플러그인 로직 확인
```

## 저장소 구조

| 폴더 | 내용 |
| --- | --- |
| `engine` | 검색 엔진(`no_std` Rust). 플러그인과 테스트가 함께 씀 |
| `plugin` | Forge 플러그인: 화면, 입력, 게임 데이터 읽기, 다른 모드 연동 |
| `plugin-hosttest` | 플러그인 로직을 PC에서 실행 |
| `data` | 게임 DB에서 만든 시뮬레이터 데이터(`simdata.bin`) |
| `assets/ui` | 플러그인 아이콘 아틀라스에 들어가는 그림 |
| `tools` | 빌드 스크립트, 데이터 생성 도구, 전체 언어의 화면 문구(`tools/strings.py`) |
| `vendor/forge-rs` | 내장한 Forge SDK |

## 라이선스

MIT License (원작: Copyright (c) 2026 ZellD-0371). `LICENSE`와, 외부 구성 요소·게임 데이터에 관한 `THIRD_PARTY_NOTICES.md`를 참고하세요. `tools/elf_to_nro.py`는 [MHGU Utility](https://github.com/Rikiar73574/mhgu-utility)(MIT, Riccardo Testa)에서 가져왔습니다.

Monster Hunter는 CAPCOM의 상표이며, 이 프로젝트는 CAPCOM과 관계없습니다.
