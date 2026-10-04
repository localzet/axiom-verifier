# axiom-verifier v0.2.0

Verifier-facing граница доверия. v0.2 разделяет **производство доказательства** (`axiom-symbolic`, позднее Z3/Lean/zkVM
backends) и **принятие receipt**. Rust-gate проверяет привязку артефактов, verdict и заявленную область корректности до
того, как runtime сможет доверять receipt.

## Связанные исследования

Этот компонент входит в исследовательский проект [Axiom](https://github.com/localzet/axiom-stack). Все компоненты собраны по теме [localzet-axiom](https://github.com/topics/localzet-axiom). Основной язык документации — русский. Исследовательские результаты и ограничения не означают готовность к промышленному применению.
