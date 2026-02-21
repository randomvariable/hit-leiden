# HIT-Leiden Mathematical Specification

Objective: Efficiently maintain Leiden communities in large dynamic graphs.

## Core Concepts

- $G=(V,E)$: Graph
- $P$: Number of hierarchical levels
- $G_p$: Supergraph at level $p$
- $\Delta G_p$: Graph updates at level $p$
- $f_p(\cdot)$: Community mapping at level $p$
- $g_p(\cdot)$: Sub-community mapping at level $p$
- $s_p^{pre}(\cdot)$: Previous sub-community mapping at level $p$
- $s_p^{cur}(\cdot)$: Current sub-community mapping at level $p$
- $\Psi_p$: Connected Component (CC) indices at level $p$
- $\gamma$: Resolution parameter

## Algorithm 6: HIT-Leiden

**Input**: $\{G_P\}$, $\Delta G$, $\{f_P(\cdot)\}$, $\{g_P(\cdot)\}$, $\{s_P^{pre}(\cdot)\}$, $\{s_P^{cur}(\cdot)\}$, $\{\Psi_P\}$, $P$, $\gamma$
**Output**: $f(\cdot)$, $\{G_P\}$, $\{f_P(\cdot)\}$, $\{g_P(\cdot)\}$, $\{s_P^{pre}(\cdot)\}$, $\{s_P^{cur}(\cdot)\}$, $\{\Psi_P\}$

1. $\Delta G_1 \leftarrow \Delta G$
2. **for** $p$ from 1 to $P$ **do**
3.   $G_p \leftarrow G_p \oplus \Delta G_p$
4.   $f_p(\cdot), \Psi, B_p, K \leftarrow \text{inc-movement}(G_p, \Delta G_p, f_p(\cdot), s_p^{cur}(\cdot), \Psi, \gamma)$
5.   $s_p^{cur}(\cdot), \Psi, R_p \leftarrow \text{inc-refinement}(G_p, f_p(\cdot), s_p^{cur}(\cdot), \Psi, K, \gamma)$
6.   **if** $p < P$ **then**
7.     $\Delta G_{p+1}, s_p^{pre}(\cdot) \leftarrow \text{inc-aggregation}(G_p, \Delta G_p, s_p^{pre}(\cdot), s_p^{cur}(\cdot), R_p)$
8. $\{f_P(\cdot)\} \leftarrow \text{def-update}(\{f_P(\cdot)\}, \{s_P^{cur}(\cdot)\}, \{B_P\}, P)$
9. $\{g_P(\cdot)\} \leftarrow \text{def-update}(\{g_P(\cdot)\}, \{s_P^{cur}(\cdot)\}, \{R_P\}, P)$
10. $f(\cdot) \leftarrow g_1(\cdot)$
11. **return** $f(\cdot)$, $\{G_P\}$, $\{f_P(\cdot)\}$, $\{g_P(\cdot)\}$, $\{s_P^{pre}(\cdot)\}$, $\{s_P^{cur}(\cdot)\}$, $\{\Psi_P\}$
