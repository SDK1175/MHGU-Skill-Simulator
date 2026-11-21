<p align="center">
  <a href="README.md">English</a> | <strong>한국어</strong>
</p>

<h3 align="center">스크린샷</h3>

<p align="center">
  <a href="docs/screenshots.md">스크린샷 보기</a>
</p>

# MHGU Skill Simulator

Monster Hunter Generations Ultimate에 게임 안에서 쓰는 방어구 스킬 시뮬레이터를 추가하는 Forge 모드입니다. 

원하는 스킬을 고르면, 실제로 가진 방어구·호석·장식품으로 그 스킬이 발동하는 조합을 찾아 줍니다. 

가진 호석으로 만들 수 없으면 필요한 호석과, 감정 한 번에 그 호석이 나올 확률(추정)을 보여 줍니다.

## 지원 환경

- MHGU 글로벌 버전 1.4.0
- Forge 플러그인을 지원하는 MHGU Recomp 혹은 Forge 1.5.1이 설치된 에뮬레이터

## 주요 기능

- 목표 스킬 최대 8개로 검색, 장비 상자·호석·현재 장비를 게임에서 직접 읽음
- 가진 호석으로 안 될 때 미보유 호석까지 검색하고, 감정 1회당 확률(추정) 표시
- 검색 결과: 추천 세트, 남는 슬롯, 방어력, 속성 내성, 장식품 배치, 발동 스킬, 새로 만들 부위와 생산 소재·금액
- 같은 스킬이 나오는 부위별 대체 방어구, 현재 장비와 비교, 세트에 더 띄울 수 있는 추가 스킬
- 조건에 맞는 세트가 없으면 어떤 조건 하나를 바꾸면 찾을 수 있는지 알려 줌
- 9개 언어: English, 简体中文, 繁體中文, 日本語, 한국어, Deutsch, Français, Italiano, Español
- Nintendo·Xbox·PlayStation 버튼 아이콘, Recomp에서 마우스·키보드 지원
- Better Skillinfo(고른 항목의 스킬 정보), Wishlist(세트·부위 찜 등록)가 설치되어 있으면 함께 동작

## 설치

1. 게임을 종료합니다.
2. `MHGU-Skill-Simulator_vX.X.X.zip`의 압축을 풉니다.
3. 압축을 푼 `MHGU-Skill-Simulator` 폴더를 Recomp 실행 파일 옆의 `mods` 폴더에 넣습니다.
4. 게임을 실행합니다. 모드 관리자에서 비활성화되어 있다면 활성화합니다.

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

### 시뮬레이터 열기

캐릭터를 불러온 뒤 + 버튼으로 게임 메뉴를 엽니다. 마을과 집회소에서는 메뉴 3페이지에 **스킬 시뮬레이터**가 있습니다. 퀘스트 중에는 메뉴에 없으므로 Forge 설정의 **열기** 버튼을 쓰세요.

Forge 설정의 **열기** 버튼으로도 열 수 있습니다.

## 설정

Insert 키로 Forge 설정을 열고 `Plugin UI > Skill Simulator`로 이동합니다.

| 설정 | 선택지 |
| --- | --- |
| Language | 지원하는 9개 언어 중 선택 (기본값 English) |
| Button layout | Nintendo / Xbox / PlayStation |
| 로그 기록 활성화 | 문제 보고용으로 검색 조건과 프레임 시간을 `log.txt`에 기록 (기본값 꺼짐) |

## 저장 데이터

설정과 캐릭터별 데이터는 게임의 현재 세이브 폴더 안에 별도 INI 파일로 저장됩니다.

| 파일 | 내용 |
| --- | --- |
| `skill_simulator.ini` | 언어, 버튼 배치, 로그 기록 |
| `skill_simulator_character_1.ini` | 첫 번째 캐릭터 슬롯의 마지막 조건, 저장한 세트, 제외한 방어구, 최근 스킬 |
| `skill_simulator_character_2.ini` | 두 번째 캐릭터 슬롯의 같은 내용 |
| `skill_simulator_character_3.ini` | 세 번째 캐릭터 슬롯의 같은 내용 |

Windows Recomp의 기본 경로는 `%APPDATA%\MHGURecomp\save\save_0\`입니다. 환경에서 세이브 경로를 따로 지정한 경우 그 경로를 사용합니다. 모드를 업데이트해도 이 파일들은 교체되지 않습니다.

## 참고

- 호석 확률은 호석 생성표로 계산한 추정값이며, 게임의 정확한 확률이 아닙니다.
- 설정과 저장 데이터는 세이브 폴더별로 따로 저장되며, 창은 캐릭터를 불러온 뒤에만 열립니다.

## 소스에서 빌드하기

필요한 도구: `rust-src`가 포함된 Rust nightly, Forge Cargo 빌드 도구(`cargo forge`), devkitARM, Python 3.10, Git Bash.

1. `tools/local.example.sh`를 `tools/local.sh`로 복사하고 devkitARM 경로(필요하면 설치할 게임 mods 폴더)를 적습니다.
2. 빌드합니다.

```bash
bash tools/build_plugin.sh
```

빌드하면 `plugin/skill_simulator.nro`와 설치 폴더 `dist/MHGU-Skill-Simulator`가 만들어집니다. `plugin/src/generated`의 생성 표와 `plugin/assets`의 파일은 저장소에 들어 있어서 게임 데이터 없이도 빌드됩니다. 게임 DB(`../Database/mhxx_all_tables.json`)가 저장소 옆에 있을 때만 다시 생성하며, 이때는 Python 패키지 Pillow와 fontTools도 필요합니다.

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

MIT License. `LICENSE`와, 외부 구성 요소·게임 데이터에 관한 `THIRD_PARTY_NOTICES.md`를 참고하세요.
