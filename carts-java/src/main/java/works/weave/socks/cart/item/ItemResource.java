package works.weave.socks.cart.item;

import works.weave.socks.cart.entities.Item;
import works.weave.socks.cart.repositories.ItemRepository;

import java.util.function.Supplier;

public class ItemResource implements Resource<Item> {
  private final ItemRepository itemRepository;
  private Supplier<Item> item;
  private final int customerId;
  private int itemId;

  public ItemResource(ItemRepository itemRepository, int customerId) {
    this.itemRepository = itemRepository;
    this.customerId = customerId;
  }

  public ItemResource(ItemRepository itemRepository, int customerId, int itemId) {
    this.itemRepository = itemRepository;
    this.customerId = customerId;
    this.itemId = itemId;
  }

  public ItemResource(ItemRepository itemRepository, Supplier<Item> item, int customerId) {
    this.itemRepository = itemRepository;
    this.item = item;
    this.customerId = customerId;
  }

  public Runnable destroyByCustomerId() {
    return () -> itemRepository.deleteByCustomerId(customerId);
  }

  @Override
  public Runnable destroy() {
    return () -> itemRepository.deleteByCustomerIdAndItemId(
            customerId,
            itemId
    );
  }

  @Override
  public Supplier<Item> value() {
    return null;
  }

  @Override
  public Supplier<Item> create() {
    return () -> {
      try {
        return itemRepository.save(new Item(
                customerId,
                item.get().getItemId(),
                item.get().getQuantity(),
                item.get().getPrice()
        ));
      } catch (NullPointerException e) {
        return itemRepository.save(new Item(
                item.get().cartId(),
                item.get().getItemId(),
                0,
                0
        ));
      }
    };
  }

  @Override
  public Runnable update() {
    return () -> itemRepository.updateAllByCustomerId(
            customerId,
            item.get().getItemId(),
            item.get().getQuantity(),
            item.get().getPrice()
    );
  }
}
