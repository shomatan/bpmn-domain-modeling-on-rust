# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# BPMN 2.0 Rust Engine

## 🎯 プロジェクト概要

BPMN 2.0仕様に準拠したビジネスプロセス実行エンジンをRustで実装する。型安全性を最優先とし、代数的データ型(ADT)とNonEmptyListを活用して、コンパイル時に多くのエラーを防ぐ設計を行う。

## 📋 実装要件

### 必須要件
- **BPMN 2.0準拠**: OMG標準仕様に従う
- **型安全性**: 不正な状態を作れない設計
- **ゼロコストアブストラクション**: Rustの特性を活かした高性能実装
- **エラーハンドリング**: Result型による明示的なエラー処理

### 設計原則
1. **Make Invalid States Unrepresentable**: 不正な状態を型で表現不可能にする
2. **Parse, Don't Validate**: 検証ではなく型変換で安全性を保証
3. **Explicit over Implicit**: 暗黙的な動作を避ける

## ⚠️ 重要な教訓：関数型ドメインモデリングを決して妥協しない

### 反省事例：Process設計での失敗
**間違った実装**:
```rust
pub struct Process {
    pub start_events: Vec<StartEvent>, // ❌ 空のプロセス作成可能
}
```

**正しい実装**:
```rust
pub struct Process {
    pub start_events: NonEmptyVec<StartEvent>, // ✅ 必ず1つ以上を型で保証
}
```

### 教訓
- **一時的な便利さに惑わされない**: `Vec`は書きやすいが型安全性を犠牲にする
- **不正な状態を作れない設計**: BPMNプロセスは必ずStartEventが必要 → NonEmptyVecで保証
- **コンパイル時チェック > 実行時チェック**: 型で制約を表現し、バグを事前に防ぐ
- **原則に忠実であること**: 関数型ドメインモデリングの原則は必ず守る

## 🏗️ プロジェクト構造

```
bpmn-engine/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # ライブラリのエントリポイント
│   ├── core/
│   │   ├── mod.rs
│   │   ├── definitions.rs     # Definitions (ルート要素)
│   │   ├── process.rs         # Process定義
│   │   └── flow_element.rs    # FlowElement階層
│   ├── elements/
│   │   ├── mod.rs
│   │   ├── event.rs           # Event全種類
│   │   ├── activity.rs        # Activity/Task
│   │   ├── gateway.rs         # Gateway全種類
│   │   └── flow.rs            # SequenceFlow/MessageFlow
│   ├── types/
│   │   ├── mod.rs
│   │   ├── non_empty.rs       # NonEmptyVec実装
│   │   ├── ids.rs             # NewType Pattern IDs
│   │   └── expression.rs      # 条件式/スクリプト
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── executor.rs        # プロセス実行エンジン
│   │   ├── token.rs           # Token管理
│   │   ├── instance.rs        # ProcessInstance
│   │   └── scheduler.rs       # Timer/Event scheduler
│   ├── validation/
│   │   ├── mod.rs
│   │   ├── structure.rs       # 構造検証
│   │   └── rules.rs           # BPMN制約検証
│   ├── builder/
│   │   ├── mod.rs
│   │   └── process_builder.rs # Phantom Type Builder
│   └── io/
│       ├── mod.rs
│       ├── xml.rs             # BPMN XML import/export
│       └── json.rs            # JSON serialization
├── tests/
│   ├── integration/
│   └── fixtures/              # テスト用BPMNファイル
└── examples/
    ├── simple_process.rs
    └── order_process.rs
```

## 📝 実装順序と詳細

### Phase 1: 基礎型定義 (最優先)

#### 1.1 NonEmptyVec実装
```rust
// src/types/non_empty.rs
pub struct NonEmptyVec<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    pub fn new(head: T) -> Self { /* 実装 */ }
    pub fn first(&self) -> &T { &self.head } // 必ず存在
    pub fn push(&mut self, value: T) { /* 実装 */ }
    pub fn len(&self) -> usize { 1 + self.tail.len() }
}
```

#### 1.2 ID型定義 (NewType Pattern)
```rust
// src/types/ids.rs
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ProcessId(String);

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ActivityId(String);

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct EventId(String);

// 他のID型も同様に定義
```

### Phase 2: Core要素実装

#### 2.1 FlowNode階層
```rust
// src/core/flow_element.rs
pub trait FlowElement {
    fn id(&self) -> &str;
    fn name(&self) -> Option<&str>;
}

pub trait FlowNode: FlowElement {
    fn incoming(&self) -> &[FlowId];
    fn outgoing(&self) -> &[FlowId];
}
```

#### 2.2 Event ADT実装
```rust
// src/elements/event.rs
pub enum EventDefinition {
    None,
    Message(MessageId),
    Timer(TimerDefinition),
    Error(Option<ErrorCode>),
    Signal(SignalId),
    Conditional(Expression),
    // 全13種類を実装
}

pub enum Event {
    Start {
        id: EventId,
        definition: EventDefinition,
        is_interrupting: bool,
    },
    IntermediateCatch {
        id: EventId,
        definition: EventDefinition,
        attached_to: Option<ActivityId>,
        cancel_activity: bool,
    },
    IntermediateThrow {
        id: EventId,
        definition: EventDefinition,
    },
    End {
        id: EventId,
        definition: EventDefinition,
    },
}

// 組み合わせ制約の検証
impl Event {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Event::Start { definition: EventDefinition::Error(_), .. } => {
                Err(ValidationError::InvalidEventCombination)
            },
            // 他の無効な組み合わせも検証
            _ => Ok(())
        }
    }
}
```

### Phase 3: Process構造

#### 3.1 Process定義（制約付き）
```rust
// src/core/process.rs
pub struct Process {
    id: ProcessId,
    name: Option<String>,
    is_executable: bool,
    
    // 最低1つのStartEventを保証
    start_events: NonEmptyVec<StartEvent>,
    
    // その他の要素
    intermediate_events: Vec<IntermediateEvent>,
    end_events: Vec<EndEvent>,
    activities: Vec<Activity>,
    gateways: Vec<Gateway>,
    
    // 最低1つのフローを保証
    sequence_flows: NonEmptyVec<SequenceFlow>,
    
    // データ要素
    data_objects: Vec<DataObject>,
}
```

### Phase 4: Builder実装

#### 4.1 Phantom Type Builder
```rust
// src/builder/process_builder.rs
use std::marker::PhantomData;

pub struct NoStart;
pub struct HasStart;
pub struct NoEnd;
pub struct HasEnd;

pub struct ProcessBuilder<S, E> {
    id: ProcessId,
    elements: Vec<FlowElement>,
    _phantom: PhantomData<(S, E)>,
}

impl ProcessBuilder<NoStart, NoEnd> {
    pub fn new(id: ProcessId) -> Self { /* 実装 */ }
}

impl<E> ProcessBuilder<NoStart, E> {
    pub fn with_start(self, event: StartEvent) -> ProcessBuilder<HasStart, E> {
        // 型を変換してStartEventの存在を保証
    }
}

impl<S> ProcessBuilder<S, NoEnd> {
    pub fn with_end(self, event: EndEvent) -> ProcessBuilder<S, HasEnd> {
        // 型を変換してEndEventの存在を保証
    }
}

// StartとEndが揃った時のみbuild可能
impl ProcessBuilder<HasStart, HasEnd> {
    pub fn build(self) -> Result<Process, BuildError> {
        // 検証してProcess生成
    }
}
```

### Phase 5: Gateway実装

```rust
// src/elements/gateway.rs
pub enum Gateway {
    Exclusive {
        id: GatewayId,
        incoming: NonEmptyVec<FlowId>,
        outgoing: Vec2<FlowId>, // 最低2つの出力
        default: Option<FlowId>,
    },
    Parallel {
        id: GatewayId,
        incoming: NonEmptyVec<FlowId>,
        outgoing: Vec2<FlowId>,
    },
    // 他のGatewayも実装
}

// 最低2要素を保証するVec2
pub struct Vec2<T> {
    first: T,
    second: T,
    rest: Vec<T>,
}
```

### Phase 6: 実行エンジン

```rust
// src/engine/executor.rs
pub struct ProcessEngine {
    processes: HashMap<ProcessId, Process>,
    instances: HashMap<InstanceId, ProcessInstance>,
    scheduler: EventScheduler,
}

impl ProcessEngine {
    pub fn deploy_process(&mut self, process: Process) -> Result<()> { /* 実装 */ }
    pub fn start_process(&mut self, process_id: &ProcessId) -> Result<InstanceId> { /* 実装 */ }
    pub fn execute_step(&mut self, instance_id: &InstanceId) -> Result<()> { /* 実装 */ }
}

// Token-based実行
pub struct Token {
    id: TokenId,
    location: FlowNodeId,
    state: TokenState,
}
```

## 🧪 テスト戦略

### ユニットテスト
各モジュールに対して：
- 正常系の動作確認
- エラーケースの検証
- 境界値テスト

### 統合テスト
```rust
// tests/integration/process_execution.rs
#[test]
fn test_simple_sequential_process() {
    let process = ProcessBuilder::new("test_process".into())
        .with_start(StartEvent::none())
        .add_task(UserTask::new("task1"))
        .with_end(EndEvent::none())
        .connect("start", "task1")
        .connect("task1", "end")
        .build()
        .unwrap();
    
    let mut engine = ProcessEngine::new();
    engine.deploy_process(process).unwrap();
    
    let instance_id = engine.start_process(&"test_process".into()).unwrap();
    // 実行とアサーション
}
```

### プロパティベーステスト
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn process_always_has_start_event(process in any_valid_process()) {
        assert!(!process.start_events.is_empty());
    }
}
```

## 🔍 検証ルール

### コンパイル時検証（型システムで保証）
- [ ] Processは最低1つのStartEvent
- [ ] NonEmptyVecで空のコレクションを防ぐ
- [ ] Phantom TypeでBuilder制約
- [ ] Event組み合わせの妥当性

### 実行時検証
- [ ] グラフの接続性（disconnected nodeがない）
- [ ] デッドロック検出
- [ ] 無限ループ検出
- [ ] Gateway条件の完全性

## 📊 パフォーマンス目標

- **プロセス定義のパース**: < 100ms (10MB XMLファイル)
- **インスタンス作成**: < 1ms
- **Token移動**: < 10μs
- **同時実行インスタンス**: 10,000+

## 🚫 アンチパターンと回避策

### 避けるべきこと
1. **Option<Vec<T>>** → `Vec<T>`を使う（空=None相当）
2. **String型のID** → NewType Patternで型安全に
3. **実行時の型チェック** → ADTとパターンマッチで静的解決
4. **unwrap()の多用** → Result型で明示的エラー処理

### 推奨パターン
1. **Builder Pattern** → 段階的な制約充足
2. **Visitor Pattern** → プロセス要素の走査
3. **State Pattern** → イベント状態管理
4. **Strategy Pattern** → Gateway分岐ロジック

## 🛠️ 依存ライブラリ

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
quick-xml = { version = "0.31", features = ["serialize"] }
petgraph = "0.6"
chrono = "0.4"
uuid = { version = "1.6", features = ["v4", "serde"] }
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"

[dev-dependencies]
proptest = "1.4"
criterion = "0.5"
```

## 📈 完了基準

### MVP (Minimum Viable Product)
- [ ] 基本的なEvent/Activity/Gateway実装
- [ ] 単純な順次プロセスの実行
- [ ] XML import/export
- [ ] 基本的な検証

### 完全実装
- [ ] 全Event種別（13種類×4位置）
- [ ] 全Gateway種別と分岐/合流ロジック
- [ ] SubProcess/CallActivity
- [ ] Compensation/Transaction
- [ ] Timer scheduling
- [ ] 並列実行とToken管理

## 💡 実装のヒント

1. **まずNonEmptyVecから実装** - 多くの場所で使用される
2. **Event組み合わせマトリックスを常に参照** - 無効な組み合わせを作らない
3. **Tokenの動きをシミュレート** - デバッグ時に視覚化すると理解しやすい
4. **小さく始めて段階的に拡張** - まずStartEvent→Activity→EndEventの単純フローから

## 🤝 コーディング規約

- **命名規則**: Rust標準（snake_case for functions, CamelCase for types）
- **エラー処理**: 全てResult型で返す、panicは避ける
- **ドキュメント**: 全public APIにdoc comment
- **テスト**: 各関数に最低1つのテスト
- **フォーマット**: `cargo fmt`でフォーマット
- **リント**: `cargo clippy`で警告をゼロに

---

**注意**: このプロジェクトはBPMN 2.0の完全準拠を目指しています。実装時は常にOMG仕様書を参照し、仕様から逸脱しないよう注意してください。

---
# BPMN 2.0 Rust Engine - 実装タスクリスト

## 📅 プロジェクト概要
- **開始日**: ___________
- **目標完了日**: ___________
- **総タスク数**: 120
- **優先度**: 🔴 必須 / 🟡 重要 / 🟢 追加機能

---

## Phase 0: プロジェクト初期化 (2日)

### 環境構築
- [ ] 🔴 Rustプロジェクト作成 (`cargo new bpmn-engine --lib`)
- [ ] 🔴 `.gitignore`設定
- [ ] 🔴 `Cargo.toml`の依存関係設定
- [ ] 🔴 CI/CD設定 (GitHub Actions)
- [ ] 🟡 pre-commitフック設定 (fmt, clippy)
- [ ] 🟡 README.md作成

### プロジェクト構造
- [ ] 🔴 `src/core/`ディレクトリ作成
- [ ] 🔴 `src/elements/`ディレクトリ作成
- [ ] 🔴 `src/types/`ディレクトリ作成
- [ ] 🔴 `src/engine/`ディレクトリ作成
- [ ] 🔴 `src/validation/`ディレクトリ作成
- [ ] 🔴 `src/builder/`ディレクトリ作成
- [ ] 🔴 `src/io/`ディレクトリ作成
- [ ] 🔴 各モジュールの`mod.rs`作成

---

## Phase 1: 基礎型システム (3日)

### NonEmptyVec実装 [`src/types/non_empty.rs`]
- [ ] 🔴 `NonEmptyVec<T>`構造体定義
- [ ] 🔴 `new()`コンストラクタ実装
- [ ] 🔴 `first()`, `last()`メソッド実装
- [ ] 🔴 `push()`, `len()`メソッド実装
- [ ] 🔴 `iter()`イテレータ実装
- [ ] 🔴 `From<T>`トレイト実装
- [ ] 🟡 `IntoIterator`トレイト実装
- [ ] 🔴 単体テスト作成（最低10ケース）

### Vec2実装 [`src/types/vec2.rs`]
- [ ] 🔴 `Vec2<T>`構造体定義（最低2要素保証）
- [ ] 🔴 `new(first, second)`コンストラクタ
- [ ] 🔴 `push()`メソッド実装
- [ ] 🔴 イテレータ実装
- [ ] 🔴 単体テスト作成

### ID型定義 [`src/types/ids.rs`]
- [ ] 🔴 `ProcessId`型定義
- [ ] 🔴 `ActivityId`型定義
- [ ] 🔴 `EventId`型定義
- [ ] 🔴 `GatewayId`型定義
- [ ] 🔴 `FlowId`型定義
- [ ] 🔴 `InstanceId`型定義
- [ ] 🔴 `TokenId`型定義
- [ ] 🟡 各ID型に`new()`, `generate()`実装
- [ ] 🟡 Display, Debug, Serialize, Deserializeトレイト実装

### Expression型 [`src/types/expression.rs`]
- [ ] 🔴 `Expression`構造体定義
- [ ] 🔴 `Language`列挙型（JavaScript, FEEL等）
- [ ] 🟡 基本的な評価機能
- [ ] 🟢 スクリプトエンジン統合

---

## Phase 2: Core抽象層 (2日)

### FlowElement階層 [`src/core/flow_element.rs`]
- [ ] 🔴 `FlowElement`トレイト定義
- [ ] 🔴 `FlowNode`トレイト定義（FlowElementを継承）
- [ ] 🔴 `incoming()`, `outgoing()`メソッド定義
- [ ] 🔴 基本実装とテスト

### Definitions [`src/core/definitions.rs`]
- [ ] 🔴 `BpmnDefinitions`構造体定義
- [ ] 🔴 `targetNamespace`等のメタデータ
- [ ] 🟡 グローバル要素管理（Message, Error, Signal）

### Process [`src/core/process.rs`]
- [ ] 🔴 `Process`構造体定義
- [ ] 🔴 `start_events: NonEmptyVec<StartEvent>`フィールド
- [ ] 🔴 `sequence_flows: NonEmptyVec<SequenceFlow>`フィールド
- [ ] 🔴 その他の要素コレクション定義
- [ ] 🔴 基本的なgetter/setter
- [ ] 🟡 要素の追加/削除メソッド

---

## Phase 3: Event実装 (5日)

### EventDefinition [`src/elements/event/definition.rs`]
- [ ] 🔴 `EventDefinition`列挙型（13種類）
- [ ] 🔴 `None`定義
- [ ] 🔴 `Message(MessageId)`定義
- [ ] 🔴 `Timer(TimerDefinition)`定義
- [ ] 🔴 `Error(Option<ErrorCode>)`定義
- [ ] 🔴 `Signal(SignalId)`定義
- [ ] 🔴 `Conditional(Expression)`定義
- [ ] 🟡 `Escalation`定義
- [ ] 🟡 `Compensation`定義
- [ ] 🟡 `Cancel`定義
- [ ] 🟡 `Terminate`定義
- [ ] 🟢 `Link`定義
- [ ] 🟢 `Multiple`定義
- [ ] 🟢 `ParallelMultiple`定義

### Timer定義 [`src/elements/event/timer.rs`]
- [ ] 🔴 `TimerDefinition`列挙型
- [ ] 🔴 `TimeDate(DateTime<Utc>)`実装
- [ ] 🔴 `TimeDuration(Duration)`実装
- [ ] 🔴 `TimeCycle`実装（周期的実行）
- [ ] 🔴 ISO 8601パーサー実装
- [ ] 🔴 Timer評価ロジック

### Event本体 [`src/elements/event/mod.rs`]
- [ ] 🔴 `Event`列挙型（Start/Intermediate/End）
- [ ] 🔴 `StartEvent`構造の実装
- [ ] 🔴 `IntermediateCatchEvent`実装
- [ ] 🔴 `IntermediateThrowEvent`実装
- [ ] 🔴 `EndEvent`実装
- [ ] 🔴 組み合わせ制約検証（validate()）
- [ ] 🟡 `BoundaryEvent`特別処理
- [ ] 🔴 各Event種別の単体テスト

---

## Phase 4: Activity実装 (4日)

### Task種別 [`src/elements/activity/task.rs`]
- [ ] 🔴 `Task`列挙型定義
- [ ] 🔴 `UserTask`実装
- [ ] 🔴 `ServiceTask`実装
- [ ] 🔴 `ScriptTask`実装
- [ ] 🟡 `BusinessRuleTask`実装
- [ ] 🟡 `SendTask`実装
- [ ] 🟡 `ReceiveTask`実装
- [ ] 🟢 `ManualTask`実装

### SubProcess [`src/elements/activity/subprocess.rs`]
- [ ] 🔴 `SubProcess`構造体定義
- [ ] 🔴 内部FlowElement管理
- [ ] 🟡 `EventSubProcess`実装
- [ ] 🟢 `TransactionSubProcess`実装
- [ ] 🟢 `AdHocSubProcess`実装

### Activity共通 [`src/elements/activity/mod.rs`]
- [ ] 🔴 `Activity`列挙型（Task/SubProcess/CallActivity）
- [ ] 🔴 Loop特性定義（StandardLoop/MultiInstance）
- [ ] 🟡 `CallActivity`実装
- [ ] 🔴 単体テスト

---

## Phase 5: Gateway実装 (3日)

### Gateway種別 [`src/elements/gateway.rs`]
- [ ] 🔴 `Gateway`列挙型定義
- [ ] 🔴 `ExclusiveGateway`実装（default flow含む）
- [ ] 🔴 `ParallelGateway`実装
- [ ] 🔴 `InclusiveGateway`実装
- [ ] 🟡 `EventBasedGateway`実装
- [ ] 🟢 `ComplexGateway`実装
- [ ] 🔴 `GatewayDirection`列挙型（Converging/Diverging）
- [ ] 🔴 分岐ロジック実装
- [ ] 🔴 合流ロジック実装
- [ ] 🔴 各Gateway種別のテスト

---

## Phase 6: Flow実装 (2日)

### SequenceFlow [`src/elements/flow/sequence.rs`]
- [ ] 🔴 `SequenceFlow`構造体定義
- [ ] 🔴 `source_ref`, `target_ref`フィールド
- [ ] 🔴 `condition_expression`フィールド
- [ ] 🔴 条件評価ロジック
- [ ] 🟡 デフォルトフロー処理

### MessageFlow [`src/elements/flow/message.rs`]
- [ ] 🟡 `MessageFlow`構造体定義
- [ ] 🟡 Participant間接続検証
- [ ] 🟡 メッセージ定義参照

### DataAssociation [`src/elements/flow/data.rs`]
- [ ] 🟢 `DataInputAssociation`実装
- [ ] 🟢 `DataOutputAssociation`実装

---

## Phase 7: Builder Pattern (3日)

### ProcessBuilder [`src/builder/process_builder.rs`]
- [ ] 🔴 `ProcessBuilder<S, E>`構造体定義
- [ ] 🔴 Phantom Type状態（NoStart, HasStart, NoEnd, HasEnd）
- [ ] 🔴 `new()`コンストラクタ
- [ ] 🔴 `with_start()`メソッド（型状態遷移）
- [ ] 🔴 `with_end()`メソッド（型状態遷移）
- [ ] 🔴 `add_activity()`メソッド
- [ ] 🔴 `add_gateway()`メソッド
- [ ] 🔴 `connect()`メソッド（要素接続）
- [ ] 🔴 `validate()`メソッド
- [ ] 🔴 `build()`メソッド（最終構築）
- [ ] 🔴 エラーハンドリング
- [ ] 🔴 統合テスト

---

## Phase 8: Validation (3日)

### 構造検証 [`src/validation/structure.rs`]
- [ ] 🔴 グラフ接続性検証（disconnected nodes検出）
- [ ] 🔴 開始/終了ノード存在確認
- [ ] 🔴 循環参照検出
- [ ] 🟡 デッドロック検出
- [ ] 🟡 到達不可能ノード検出

### BPMN制約検証 [`src/validation/rules.rs`]
- [ ] 🔴 Event組み合わせ制約チェック
- [ ] 🔴 Gateway分岐制約（最低2出力）
- [ ] 🔴 MessageFlow制約（異なるPool間のみ）
- [ ] 🔴 StartEvent制約（incoming無し）
- [ ] 🔴 EndEvent制約（outgoing無し）
- [ ] 🟡 EventBasedGateway後続制約
- [ ] 🔴 検証レポート生成

---

## Phase 9: 実行エンジン (5日)

### Token管理 [`src/engine/token.rs`]
- [ ] 🔴 `Token`構造体定義
- [ ] 🔴 `TokenState`列挙型（Waiting/Active/Completed）
- [ ] 🔴 Token生成/消費ロジック
- [ ] 🔴 Token移動ロジック
- [ ] 🔴 Token分裂（Parallel Gateway）
- [ ] 🔴 Token合流

### ProcessInstance [`src/engine/instance.rs`]
- [ ] 🔴 `ProcessInstance`構造体定義
- [ ] 🔴 インスタンス状態管理
- [ ] 🔴 変数スコープ管理
- [ ] 🔴 実行履歴記録
- [ ] 🟡 サスペンド/レジューム機能

### Executor [`src/engine/executor.rs`]
- [ ] 🔴 `ProcessEngine`構造体定義
- [ ] 🔴 `deploy_process()`メソッド
- [ ] 🔴 `start_process()`メソッド
- [ ] 🔴 `execute_step()`メソッド
- [ ] 🔴 Activity実行ハンドラー
- [ ] 🔴 Gateway評価ロジック
- [ ] 🔴 Event処理
- [ ] 🟡 並列実行サポート

### Scheduler [`src/engine/scheduler.rs`]
- [ ] 🟡 `EventScheduler`実装
- [ ] 🟡 Timer Event スケジューリング
- [ ] 🟡 定期実行管理
- [ ] 🟢 非同期イベント処理

---

## Phase 10: I/O実装 (4日)

### XML Import/Export [`src/io/xml.rs`]
- [ ] 🔴 BPMN 2.0 XMLスキーマ定義
- [ ] 🔴 XMLパーサー実装（quick-xml使用）
- [ ] 🔴 Process要素パース
- [ ] 🔴 Event要素パース
- [ ] 🔴 Activity要素パース
- [ ] 🔴 Gateway要素パース
- [ ] 🔴 SequenceFlow要素パース
- [ ] 🟡 Collaboration要素パース
- [ ] 🔴 XMLエクスポート機能
- [ ] 🔴 ラウンドトリップテスト

### JSON Serialization [`src/io/json.rs`]
- [ ] 🟡 JSON Schema定義
- [ ] 🟡 Serialize実装（serde使用）
- [ ] 🟡 Deserialize実装
- [ ] 🟡 カスタムシリアライザー

---

## Phase 11: テストとドキュメント (3日)

### 統合テスト [`tests/integration/`]
- [ ] 🔴 単純順次プロセステスト
- [ ] 🔴 排他ゲートウェイテスト
- [ ] 🔴 並列ゲートウェイテスト
- [ ] 🔴 タイマーイベントテスト
- [ ] 🔴 エラーハンドリングテスト
- [ ] 🟡 SubProcessテスト
- [ ] 🟡 複雑なプロセステスト

### ドキュメント
- [ ] 🔴 API ドキュメント（rustdoc）
- [ ] 🔴 使用例（examples/）作成
- [ ] 🟡 アーキテクチャドキュメント
- [ ] 🟡 パフォーマンスベンチマーク
- [ ] 🟢 チュートリアル作成

### パフォーマンス [`benches/`]
- [ ] 🟢 プロセス解析ベンチマーク
- [ ] 🟢 実行エンジンベンチマーク
- [ ] 🟢 メモリ使用量測定

---

## Phase 12: 高度な機能 (5日)

### Compensation（補償）
- [ ] 🟢 Compensation Event実装
- [ ] 🟢 補償ハンドラー
- [ ] 🟢 トランザクションロールバック

### Collaboration
- [ ] 🟢 Participant/Pool実装
- [ ] 🟢 Lane実装
- [ ] 🟢 MessageFlow完全実装
- [ ] 🟢 Conversation実装

### Choreography
- [ ] 🟢 ChoreographyTask実装
- [ ] 🟢 参加者間相互作