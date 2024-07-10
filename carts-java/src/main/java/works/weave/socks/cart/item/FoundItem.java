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

  public FoundItem(Supplier<ItemRepository> itemRepository, int customerId) {
    this.customerId = customerId;
    this.itemRepository = itemRepository;
  }

  public FoundItem(Supplier<Item> item, Supplier<ItemRepository> itemRepository) {
    this.item = item;
    this.itemRepository = itemRepository;
  }

  public FoundItem(Supplier<Item> item, Supplier<ItemRepository> itemRepository, int customerId) {
    this.customerId = customerId;
    this.item = item;
    this.itemRepository = itemRepository;
  }

  @Override
  public Item get() {
    LOG.info("Query item by item id {}", item.get().getItemId());
    return itemRepository.get().findByItemId(item.get().getItemId());
  }

  public List<Item> getByCustomerId() {
    LOG.info("Query items by cart id {}", customerId);
    return itemRepository.get().findAllByCustomerId(customerId);
  }

  public Item getByItemAndCustId(int itemId) {
    LOG.info("Query items by cart id {} and filter by unique item id {}", customerId, itemId);
    return getByCustomerId().stream()
                   .filter(itemEntry -> itemEntry.getItemId() == itemId)
                   .findFirst()
                   .orElseThrow(() -> new IllegalArgumentException("Cannot find item in db"));
  }
}
