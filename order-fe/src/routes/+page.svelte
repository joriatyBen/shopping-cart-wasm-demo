<script>
	let customer = {
		name: "",
		email: "",
		phone: "",
		address: "",
		city: "",
		pin: "",
	}

	let cart = [];
	
	let products = [
		{id: 1, name: "Apple", image: "https://www.applesfromny.com/wp-content/uploads/2020/05/Jonagold_NYAS-Apples2.png", price: 10, quantity: 1},
		{id: 2, name: "Orange", image: "https://5.imimg.com/data5/VN/YP/MY-33296037/orange-600x600-500x500.jpg", price: 11, quantity: 1},
		{id: 3, name: "Grapes", image: "https://www.aicr.org/wp-content/uploads/2020/01/shutterstock_533487490-640x462.jpg", price: 12, quantity: 1},
	]

	let order = {
		customer: customer,
		checkout: cart,
		orderTotal: "",
		orderState: "",
	}

	let resultWasm = null
	
	
	const addToCart = (product) => {
		for(let item of cart) {
				if(item.id === product.id) {
					product.quantity += 1
					cart = cart;
					return;
				}
		}
		cart = [...cart, product]
	}
	
	const minusItem = (product) => {
		for(let item of cart) {
				if(item.id === product.id) {
					if(product.quantity > 1 ) {
							product.quantity -= 1
							cart = cart
					} else {
							cart = cart.filter((cartItem) => cartItem != product)
					}
					return;
				}
		}
	}
	
	const plusItem = (product) => {
		for(let item of cart) {
			if(item.id === product.id) {
				item.quantity += 1
				cart = cart;
				return;
			}
		}
	}

	$: total = cart.reduce((sum, item) => sum + item.price * item.quantity, 0)
	$: order.checkout = cart;
	const checkout = async () => {
		order.orderTotal = total.toString();
		order.orderState = "Checkout";
		if (cart.length == 0) {
			alert("Cart empty - no order placed!")
		} else if (customer.name == "" || customer.email == "" 
		|| customer.phone == "" || customer.address == "" || customer.city == "" 
		|| customer.pin == "") {
			alert("Missing customer info")
		} else {
			//alert(JSON.stringify(order))
			//alert(JSON.stringify(customer))
			const res = await fetch('http://127.0.0.1:3000/checkout', {
				method: 'POST',
				body: JSON.stringify(order)
			})
		
			const json = await res.text()
			resultWasm = json
		}
	}
</script>

<p>There are {cart.length} items in your cart</p>
<div class="product-list">
	{#each products as product}
	<div>
		<div class="image" style="background-image: url({product.image})"></div>
	<h4>{product.name}</h4>
	<p>€{product.price}</p>
	<button on:click={() => addToCart(product)}>Add to cart</button>
	</div>
	{/each}
</div>

<div class="cart-list">
	{#each cart as item }
		{#if item.quantity > 0}
		<div class="cart-item">
			<img width="50" src={item.image} alt={item.name}/>
			<div>{item.quantity}
				<button on:click={() => plusItem(item)}>+</button>
				<button on:click={() => minusItem(item)}>-</button>
			</div>
			<p>€{item.price * item.quantity}</p>
		</div>
		{/if}
	{/each}
	<div class="total">
		<h4>Total: € {total}</h4>
	</div>
</div>

<div class="shipping-address">
	<div>
    	<p>Name</p>
    	<input bind:value={customer.name} />	
    	<p>Email</p>
    	<input bind:value={customer.email} />	
    	<p>Phone</p>
    	<input bind:value={customer.phone} />	
    	<p>Address</p>
    	<textarea bind:value={customer.address} cols="24" rows="6"></textarea>	
    	<p>City</p>
    	<input bind:value={customer.city} />	
    	<p>PIN</p>
    	<input bind:value={customer.pin} />

	    <div>
	    	<button class="button button-wasm" on:click={checkout}>Checkout (WASM)</button>
	    </div>

		<div class="result-wasm">
	    	<p>
	    		Result:
	    	</p>
	    	<pre>
	    	{resultWasm}
	    	</pre>	
		</div>
	
	</div>
	<div class="current-address">
		<p>{customer.name}</p>
		<p>{customer.email}</p>
		<p>{customer.phone}</p>
		<p>{customer.address}</p>
		<p>{customer.city}</p>
		<p>{customer.pin}</p>
	</div>
</div>

<style>
	.product-list, .cart-item {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
	}
	
	.image {
		height: 150px;
		width: 150px;
		background-size: contain;
		background-position: center;
		background-repeat: no-repeat;
	}
	.total {
		text-align: right;
	}
	
	.cart-list {
		border: 2px solid;
		padding: 10px;
	}

	.shipping-address {
		display: grid;
		grid-template-columns: 1fr 1fr;
		grid-gap: 1em;
	}

	.current-address {
		color: grey;
		padding: 2em;
		border: 1px dashed black
	}

	.result-wasm {
		display: grid;
		grid-template-columns: 1fr 1fr;
		grid-gap: 1em;
		padding: 2em;
		border: 2px solid #04AA6D
	}

	.button {
    	background-color: #04AA6D;
        border: none;
        color: white;
        text-align: center;
        text-decoration: none;
        display: inline-block;
        font-size: 16px;
        margin: 20px 2px;
        cursor: pointer;
}

	.button-wasm {padding: 15px 24px;}
</style>
