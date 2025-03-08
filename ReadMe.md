# XSTM
A Rust-implemented Software Transactional Memory (STM) library using TL2 (Transactional Locking II) algorithm  
一个Rust编写的STM(软件事务内存)库, 使用TL2(Transactional Locking II)算法

## 例子 Example
[fibonacci](tests/fib.rs)

## 一些限制 Limits
- 事务失败会重试, 不能在事务块中执行一些重试会导致错误的代码, 最好是执行纯函数
- 事务变量`TVar<T>`中的T必须满足`T: Copy`, 因为TL2是一个乐观并发算法，必须要求事务变量可以安全地并发读取
  - 实际上对于很多`T: Clone`的类型, 也是可以安全的用于TVar的, 但也不是所有的`T: Clone`都可以用。因为TL2算法使用的是双校验读（大概是“检查版本”-“读数据（`Copy`/`Clone`）”-“再次检查版本”）, 这种方法确实能确保在最后读到的数据是有效的。但在读的过程中（`Clone`），可能会读到出错的脏数据，如果`Clone`中使用了这些脏数据，可能会导致错误。例如`Clone`中需要根据读到`size`分配内存，而此时读到了一个错误的大小（比如-10086）, 这就会导致很严重的问题
- 事务调用`atomically`函数不能嵌套使用


## todo
- 完善测试
  - 考虑接入loom进行覆盖测试 (STM的重试机制可能会导致loom执行路径数量无限膨胀, 应该可以解决该问题)
- 进一步提升性能
  - 进一步确认性能热点
  - 使用论文中的提到的BloomFilter加速事务日志的查找
  - 提前区分读事务和写事务，让写事务不再至少需要重试一次
- 调整接口
  - 现在的`Transaction`接口有太多生命周期标注，且和STM的操作上下文强绑定，应当提供更加通用的接口，最好能支持除TL2以外的其它软件事务内存算法
  - 增加更多的事务组合操作, 例如`or_else`组合子
- 解除对`TVar<T: Copy>`的限制
  - 增加一个表达安全读取的trait，自动对所有`T: Copy`实现
- 完善文档/注释
- 支持硬件事务内存？
