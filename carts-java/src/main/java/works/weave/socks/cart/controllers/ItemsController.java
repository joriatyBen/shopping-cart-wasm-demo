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
  public List<Item> get(@PathVariable int customerId) {
    return new FoundItem(() -> itemRepository, customerId).getByCustomerId();
  }

  @ResponseStatus(HttpStatus.OK)
  @RequestMapping(value = "/items/{itemId:.*}", produces = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.GET)
  public Item get(@PathVariable int customerId, @PathVariable int itemId) {
    return new FoundItem(() -> itemRepository, customerId).getByItemAndCustId(itemId);
  }

  @ResponseStatus(HttpStatus.OK)
  @RequestMapping(value = "/items", produces = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.GET)
  public List<Item> getItems(@PathVariable int customerId) {
    return new FoundItem(() -> itemRepository, customerId).getByCustomerId();
  }

  @ResponseStatus(HttpStatus.CREATED)
  @RequestMapping(value = "/items", consumes = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.POST)
  public Item addToCart(@PathVariable int customerId, @RequestBody Item item) {
    try {
      FoundItem foundItem = new FoundItem(() -> item, () -> itemRepository);
      foundItem.getByItemAndCustId(item.getItemId());
      LOG.info("Found item in cart. Updating quantity {} and price {} for user: {}", item.getQuantity(), item.getPrice(), customerId);
      updateItem(customerId, item);
      item.setCustomerId(customerId);

      return item;
    } catch (RuntimeException e) {
      LOG.info("Did not find item. Creating item for user: {}, {}", customerId, item.getItemId());
      return new ItemResource(itemRepository, () -> item, customerId).create().get();
    }
  }

  @ResponseStatus(HttpStatus.ACCEPTED)
  @RequestMapping(value = "/items/{itemId:.*}", method = RequestMethod.DELETE)
  public void removeItem(@PathVariable int customerId, @PathVariable int itemId) {
    try {
      Item foundItem = new FoundItem(() -> itemRepository, customerId).getByItemAndCustId(itemId);
      LOG.info("Deleting item: {}", foundItem.getItemId());
      new ItemResource(itemRepository, customerId, itemId).destroy().run();
    } catch (IllegalArgumentException e) {
      LOG.warn("Item {} not found", itemId);
    }
  }

  @ResponseStatus(HttpStatus.ACCEPTED)
  @RequestMapping(value = "/items", method = RequestMethod.DELETE)
  public void removeItem(@PathVariable int customerId) {
    List<Item> foundItem = new FoundItem(() -> itemRepository, customerId).getByCustomerId();
    LOG.info("Deleting {} item(s) in cart for user: {}", (long) foundItem.size(), customerId);
    new ItemResource(itemRepository, customerId).destroyByCustomerId().run();
  }

  @ResponseStatus(HttpStatus.ACCEPTED)
  @RequestMapping(value = "/items", consumes = MediaType.APPLICATION_JSON_VALUE, method = RequestMethod.PATCH)
  public void updateItem(@PathVariable int customerId, @RequestBody Item item) {
    try {
      Item foundItem = new FoundItem(() -> item, () -> itemRepository, customerId).getByItemAndCustId(item.getItemId());
      LOG.info("Updating item in cart for user: {}, {}", customerId, foundItem.getItemId());
      new ItemResource(itemRepository, () -> item, customerId).update().run();
    } catch (IllegalArgumentException e) {
      //e.printStackTrace();
      LOG.warn("Cannot find item with customer_id: {}, item_id: {}", customerId, item.getItemId());
      LOG.info("Not updating item in cart");
    }
  }
}
