// A special Head tuple: HPtr: point to the head of the list. HRef: number of activate threads。
//  per-thread Handle variable: for each thread, store the snapshot of HPtr
// 每个节点都有两个字段：Next：指向列表中的下一个节点, NRef(可以访问这个节点的线程数)
// Figure 3 (b)为什么HRef
//
