# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · **🇰🇷 한국어** · [🇯🇵 日本語](README.ja.md)

공개 출처의 대상 간 관계를 정리하고 개인정보를 키로 가명 처리하는 애플리케이션입니다.

상태: **초기 개발**(사전 릴리스). 구현: Rust(핵심)와 React(창). 인터페이스와 문서의 기준 언어는 우크라이나어입니다.

ZaNytky는 **독점 소프트웨어**입니다. 저장소가 공개되어 있다고 해서 오픈소스 라이선스가 부여되는 것은 아닙니다(LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md 참조). 사용자 데이터는 저장소에 보관되지 않습니다.

> 이 번역은 자동 생성되었으며 원어민 검수를 거치지 않았습니다. 기준 텍스트는 우크라이나어입니다.

## 기능

- 시스템 키로 값(이름, 세금 번호, 전화, 이메일)의 일대일 가명을 만듭니다. 같은 값은 항상 같은 코드가 됩니다.
- 원본은 조사관의 공개 키로 봉인된 상태로만 저장되며(age), 개인 키 소유자만 열 수 있습니다.
- AI 분석 전 텍스트 비식별화와 AI용 패키지(텍스트, 지침, 메타데이터), 인식되지 않은 언급에 대한 조사관의 결정.
- AI 답변 검사, 사람의 검토, 데이터베이스(SQLite)로 "미검증" 상태로 가져오기.
- 모델: 대상, 최초 출처까지의 사슬을 가진 출처, 신뢰도·신빙성 등급을 가진 진술, 소유 구조 트리.
- 암호문으로 보호되는 Markdown, Word, PDF 조사 보고서.

## 다운로드

**다운로드: [ZaNytky v0.0.3](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.3)** (최신 사전 출시 버전)

Windows용 완성 파일(설치 없이 쓰는 휴대용 zip, .msi/.exe 설치 프로그램)은 [릴리스](https://github.com/RomanZavadaM/ZaNytky/releases) 페이지에 있습니다. 빌드는 서명되지 않았으므로 Windows가 알 수 없는 게시자 경고를 표시할 수 있습니다.

## 빠른 시작(애플리케이션)

1. 휴대용 압축 파일을 풀고 `ZaNytky.exe`를 실행합니다(설치할 것이 없습니다. `webview2` 폴더를 `ZaNytky.exe` 옆에 그대로 두세요).
2. "설정" 탭: 빈 데이터 폴더를 선택하고 "폴더 준비"를 누릅니다. 조사관 키를 만들고 그 파일을 안전한 곳에 보관합니다.
3. "문서 준비" 탭: 텍스트를 붙여넣고 "비식별화하고 AI용 패키지 만들기"를 누른 뒤 텍스트와 지침을 복사해 AI에 전달합니다.
4. "AI 결과" 탭: 답변(result.json)을 붙여넣고 검사한 뒤 검토 내용을 읽고 가져옵니다.
5. "데이터베이스"와 "보고서" 탭: 진술을 보고 확인하거나 반박하며 보고서를 만듭니다.

## 명령줄

같은 기능을 명령으로도 쓸 수 있으며 `zanytky help`가 전체 목록을 보여줍니다.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky register name "Іваненко Петро" --home ./home
zanytky prepare doc.md --home ./home --out ./pkg --id pkg-1 --title "Title" --identity ./investigator.txt
zanytky check  result.json --package pkg-1
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
```

## 원칙

- 모든 진술에는 출처와 1차 출처까지의 연결 고리가 있습니다. 소문은 사실이 되지 않으며, 같은 소문의 반복은 하나의 1차 출처로 봅니다.
- 사람에 대한 평가성 꼬리표는 없습니다. 프로그램은 판정이 아니라 출처와 신뢰도 등급을 보여 줍니다.
- 공개 출처만 사용하며, 개인정보는 가명으로 대체되고, AI는 비식별화된 텍스트만 받습니다.

## 현재 버전(0.0.3)의 제한 사항

- 사전 출시 버전이며 설치 파일은 서명되어 있지 않습니다.
- 「문서 분석」은 현재 텍스트와 Markdown만 받습니다(.docx/.pdf 가져오기는 아직 없음).
- Word 보고서는 Microsoft Word에서, PDF는 뷰어에서 확인하지 않았습니다. 알 수 없는 이름 감지는 단순하며(대문자로 시작하는 단어) 잘못된 후보가 나올 수 있습니다.
- 데이터 폴더는 코드 폴더 밖에 두고, 패키지 식별자에 개인정보(예: 이메일 주소)를 넣지 마세요. 식별자는 비식별화 없이 AI에 전달됩니다.

## 보안

시스템 키(데이터 폴더의 `system.key`), 조사관의 개인 키, 암호문은 따로 보관하고 저장소에 절대 추가하지 마세요. 애플리케이션은 네트워크로 아무것도 보내지 않습니다.

## 문서

- [사용자 가이드](../user-guide/USER_GUIDE.ko.md)
- [LICENSE](../../LICENSE.md)
