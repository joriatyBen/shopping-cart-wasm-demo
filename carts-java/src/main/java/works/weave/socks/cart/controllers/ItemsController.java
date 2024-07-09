package works.weave.socks.cart.controllers;

import org.slf4j.Logger;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.web.bind.annotation.*;
import works.weave.socks.cart.entities.Item;
import works.weave.socks.cart.item.FoundItem;
import works.weave.socks.cart.item.ItemResource;
import works.weave.socks.cart.repositories.ItemRepository;

import java.util.List;

import static org.slf4j.LoggerFactory.getLogger;

@RestController
@RequestMapping(value = "/carts/{customerId:.*}")
public class ItemsController {
  private final Logger LOG = getLogger(getClass());

  @Autowired
  ItemRepository itemRepository;


  @ResponseStatus(HttpStatus.OK)
  @RequestMapping(produces = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.GET)
  public Item get(@PathVariable int customerId) {
    return new FoundItem(customerId, () -> itemRepository).getByItemId();
  }

  @ResponseStatus(HttpStatus.OK)
  @RequestMapping(value = "/items/{itemId:.*}", produces = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.GET)
  public Item get(@PathVariable int customerId, @PathVariable int itemId) {
    return new FoundItem(customerId, () -> itemRepository).getByItemId(itemId);
  }

  @ResponseStatus(HttpStatus.OK)
  @RequestMapping(value = "/items", produces = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.GET)
  public List<Item> getItems(@PathVariable int customerId) {
    return new FoundItem(customerId, () -> itemRepository).getByCustomerId();
  }

  @ResponseStatus(HttpStatus.CREATED)
  @RequestMapping(value = "/items", consumes = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.POST)
  public Item addToCart(@PathVariable int customerId, @RequestBody Item item) {
    try {
      FoundItem foundItem = new FoundItem(() -> item, () -> itemRepository);
      updateItem(customerId, foundItem.get());
      LOG.debug("Found item in cart. Incrementing for user: {}, {}", customerId, item);
      return item;
    } catch (NullPointerException e) {
      LOG.debug("Did not find item. Creating item for user: {}, {}", customerId, item);
      return new ItemResource(itemRepository, () -> item, customerId).create().get();
    }
  }

  @ResponseStatus(HttpStatus.ACCEPTED)
  @RequestMapping(value = "/items/{itemId:.*}", method = RequestMethod.DELETE)
  public void removeItem(@PathVariable int customerId, @PathVariable int itemId) {
    LOG.debug("Deleting item: {}", itemId);
    new ItemResource(itemRepository, customerId, itemId).destroy().run();
  }

  @ResponseStatus(HttpStatus.ACCEPTED)
  @RequestMapping(value = "/items", method = RequestMethod.DELETE)
  public void removeItem(@PathVariable int customerId) {
    LOG.debug("Deleting item wit cart_id: {}", customerId);
    new ItemResource(itemRepository, customerId).destroyByCustomerId().run();
  }

  @ResponseStatus(HttpStatus.ACCEPTED)
  @RequestMapping(value = "/items", consumes = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.PATCH)
  public void updateItem(@PathVariable int customerId, @RequestBody Item item) {
    LOG.debug("Updating item in cart for user: {}, {}", customerId, item);
    new ItemResource(itemRepository, () -> item, customerId).update().run();
  }
}
