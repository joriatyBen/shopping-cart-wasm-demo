package works.weave.socks.cart.item;

import org.slf4j.Logger;
import works.weave.socks.cart.entities.Item;
import works.weave.socks.cart.repositories.ItemRepository;

import java.util.List;
import java.util.function.Supplier;

import static org.slf4j.LoggerFactory.getLogger;

public class FoundItem implements Supplier<Item> {
  private final Logger LOG = getLogger(getClass());
  private Supplier<Item> item;
  private final Supplier<ItemRepository> itemRepository;
  private int customerId;

  public FoundItem(int customerId, Supplier<ItemRepository> itemRepository) {
    this.customerId = customerId;
    this.itemRepository = itemRepository;
  }

  public FoundItem(Supplier<Item> item, Supplier<ItemRepository> itemRepository) {
    this.item = item;
    this.itemRepository = itemRepository;
  }

  @Override
  public Item get() {
    return itemRepository.get().findByItemId(item.get().getItemId());
  }

  public List<Item> getByCustomerId() {
    return itemRepository.get().findByCustomerId(customerId);
  }

  public Item getByItemId() {
    return getByCustomerId().stream()
                   .filter(itemEntry -> itemEntry.getItemId() == item.get().getItemId())
                   .findFirst()
                   .orElseThrow(() -> new IllegalArgumentException("Cannot find item in db"));
  }

  public Item getByItemId(int itemId) {
    return getByCustomerId().stream()
                   .filter(itemEntry -> itemEntry.getItemId() == itemId)
                   .findFirst()
                   .orElseThrow(() -> new IllegalArgumentException("Cannot find item in db"));
  }
}
