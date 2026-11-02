# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · **🇰🇷 한국어** · [🇯🇵 日本語](README.ja.md)

## 다운로드

**최신 버전: [ZaNytky v0.0.8](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.16) (사전 릴리스)**

| 플랫폼 | 파일 |
|---|---|
| Windows - 설치 프로그램 (권장) | [ZaNytky_0.0.16_Setup_Windows_x64.exe](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.16/ZaNytky_0.0.16_Setup_Windows_x64.exe) |
| Windows - MSI | [ZaNytky_0.0.16_Windows_x64.msi](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.16/ZaNytky_0.0.16_Windows_x64.msi) |
| Windows - 휴대용 압축 파일 (체험용) | [ZaNytky_0.0.16_Windows_x64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.16/ZaNytky_0.0.16_Windows_x64_Portable.zip) |
| Windows - 명령줄 | [ZaNytky_0.0.16_CLI_Windows_x64.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.16/ZaNytky_0.0.16_CLI_Windows_x64.zip) |
| macOS (Apple Silicon) | [ZaNytky_0.0.16_macOS_arm64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.16/ZaNytky_0.0.16_macOS_arm64_Portable.zip) |
| macOS (Intel) | [ZaNytky_0.0.16_macOS_x86_64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.16/ZaNytky_0.0.16_macOS_x86_64_Portable.zip) |

- 체크섬: [SHA256SUMS.txt](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.16/SHA256SUMS.txt) 및 [릴리스 페이지](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.16). 모든 릴리스: [Releases](https://github.com/RomanZavadaM/ZaNytky/releases).
- 빌드는 서명되지 않았습니다. Windows가 알 수 없는 게시자 경고를 표시할 수 있으며, macOS 빌드는 공증되지 않았고 실제 Mac에서 테스트되지 않았습니다.
- 휴대용 압축 파일은 체험용이며 어떠한 보증도 없습니다. 상시 사용에는 설치 프로그램을 사용하세요.

공개 출처에서 대상 간의 관계를 정리하는 애플리케이션: 전체 본문과 파일이 있는 출처, 인용문이 있는 진술, 관계 그래프, 대상 간 최단 경로 검색. 개인정보는 키 기반 가명으로 대체됩니다.

상태: **초기 개발 단계** (사전 릴리스 0.0.8). ZaNytky는 **독점 소프트웨어**입니다. 공개 저장소라고 해서 오픈소스 라이선스가 부여되지 않습니다 (LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). 저장소에는 사용자 데이터가 저장되지 않습니다.

> 이 번역은 기계 번역이며 원어민 검토를 거치지 않았습니다. 우크라이나어 텍스트가 기준입니다.

## 기능

- **출처:** 전체 본문과 파일(스캔, 사진, 영상)을 로컬 데이터베이스에 저장하고 제목·본문·주소로 검색합니다.
- **텔레그램 채널 가져오기** (Telegram Desktop 내보내기, JSON): 각 게시물이 본문과 첨부가 있는 출처가 됩니다.
- **출처 인용문이 있는 진술:** 종류, 역할이 있는 당사자(기존 또는 새 대상), 신빙성 등급. 항상 "미검증"입니다.
- **대상:** 진술, 출처, 소유 체인, 공통 표지(주소, 전화, 차량 번호, 계좌)가 있는 카드. 중복 병합, 관계 그래프, 두 대상 간 최단 경로.
- **가명화:** 시스템 키 기반의 일대일 코드. 원본은 조사자 키로만 봉인(age). 텍스트 비식별화와 AI용 패키지, AI 응답 검사 및 가져오기.
- **보고서:** Markdown, Word, PDF(암호문 보호). 데이터 폴더의 암호화 백업. 7개 언어.

## 빠른 시작

1. 애플리케이션을 설치합니다(또는 휴대용 압축 파일을 풀고 `ZaNytky.exe` 실행).
2. "설정": 코드 폴더 밖의 빈 데이터 폴더를 선택하고 "폴더 준비"를 누른 뒤 조사자 키를 만들어 따로 보관합니다.
3. "출처": 텔레그램 채널 내보내기를 가져오거나(상단 버튼) 출처에 본문과 파일을 직접 추가합니다.
4. 출처를 열고 진술을 만듭니다: 인용문, 당사자, 역할. "대상"과 "그래프"를 확인합니다.
5. AI 분석: "문서"(비식별화와 패키지)와 "AI 결과"(검사와 가져오기). "보고서"가 요약을 제공합니다.

## 명령줄

같은 기능을 명령으로도 쓸 수 있습니다. `zanytky help`로 전체 목록을 봅니다.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky import-telegram "ChatExport_2026-11-01" --home ./home --reliability F
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
zanytky backup --home ./home --out copy.znbak --passphrase-file ./pass.txt
```

## 원칙

- 모든 진술에는 출처와 원출처까지의 사슬이 있습니다. 소문은 사실이 되지 않으며 한 소문의 메아리는 하나의 원출처입니다.
- 사람에 대한 평가성 낙인은 없습니다. 출처와 신뢰도 등급을 보여 줄 뿐 판결은 하지 않습니다.
- 공개 출처만 사용합니다. 본문과 파일은 비식별화 없이 로컬에 보관되며 AI는 비식별화된 텍스트만 받습니다.

## 현재 버전의 제한 (0.0.16)

- 사전 릴리스: 설치 프로그램은 서명되지 않았습니다.
- PDF 가져오기와 스캔·영상의 텍스트 인식은 아직 없습니다. 진술의 인용문은 직접 붙여 넣습니다.
- Word 보고서와 PDF 보기는 Microsoft Word와 PDF 뷰어에서 확인되지 않았고 미지의 이름 인식은 단순합니다.
- 데이터 폴더는 코드 폴더 밖에 두고 패키지 식별자에 개인정보를 넣지 마세요. 식별자는 비식별화 없이 AI로 전달됩니다.

## 보안

시스템 키(데이터 폴더의 `system.key`), 조사자 개인 키, 암호문은 따로 보관하고 저장소에 절대 추가하지 마세요. 애플리케이션은 네트워크로 아무것도 보내지 않습니다.

## 문서

- [사용자 안내서](../../docs/user-guide/USER_GUIDE.ko.md)
- [릴리스 노트](../../docs/releases)
- [LICENSE](../../LICENSE.md)
