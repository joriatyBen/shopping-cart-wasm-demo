package works.weave.socks.cart.item;

import java.util.function.Supplier;

public interface Resource<T> {
  Runnable destroy();

  Supplier<T> value();

  Supplier<T> create();

  Runnable update();
}