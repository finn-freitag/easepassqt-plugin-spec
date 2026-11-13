;; A component that imports an interface outside the allow-list. Hosts MUST refuse to
;; install it; tools/check_component.py must report it as forbidden.
(component
  (import "wasi:filesystem/preopens@0.2.0" (instance))
  (import "easepass:plugin/log@1.0.0" (instance)))
